extern crate lazy_static;

extern crate mpd;
extern crate time;

use crate::action::Action;
use crate::config::*;
use crate::format::*;
use crate::util::{CachedValue, TimedValue};
use mpd::song::Song;
use mpd::status::{State, Status};
use std::collections::{BTreeMap, HashMap};
use std::net::TcpStream;
use std::process;
use std::sync::{Arc, Mutex};
use time::{get_time, Duration};

use crate::view::*;

pub type SharedModel<'m> = Arc<Mutex<Model<'m>>>;

// Static map containing the description of every action
lazy_static! {
  pub static ref ACTION_DESCRIPTION: BTreeMap<&'static str, &'static str> = {
    let mut m = BTreeMap::new();
    m.insert("execute_command", "Execute a command");
    m.insert("playlist_play", "Play the playlist");
    m.insert("playlist_pause", "Pause the playlist");
    m.insert("playlist_stop", "Stop the playlist");
    m.insert("playlist_clear", "Clear the playlist");
    m.insert("playlist_delete_items", "Delete songs from the playlist");
    m.insert("playlist_previous", "Play the playlist's previous song");
    m.insert("playlist_next", "Play the playlist's next song");
    m.insert("play_selected", "Play the selected song");
    m.insert("process_mouse", "Process mouse events");
    m.insert("resize_windows", "Resize the windows");
    m.insert("scroll_down", "Scroll down in a list");
    m.insert("scroll_up", "Scroll up in a list");
    m.insert("page_down", "Page down in a list");
    m.insert("page_up", "Page up in a list");
    m.insert("move_home", "Move to the start of a list");
    m.insert("move_end", "Move to the end of a list");
    m.insert("show_help", "Show the help view");
    m.insert("show_playlist", "Show the playlist view");
    m.insert("show_server_info", "Show the MPD server information");
    m.insert("toggle_bitrate_visibility", "Toggle the bitrate visibility");
    m.insert("toggle_random", "Toggle the \"random\" mode");
    m.insert("toggle_repeat", "Toggle the \"repeat\" mode");
    m.insert("volume_down", "Lower the volume");
    m.insert("volume_up", "Raise the volume");
    m
  };
}

fn start_client(config: &Config) -> Result<mpd::Client, mpd::error::Error> {
  mpd::Client::connect(config.socket_addr())
}

fn get_song_info(song: &Song, tag: &SongProperty) -> String {
  match *tag {
    SongProperty::Title => {
      return match song.title.as_ref() {
        Some(t) => t.clone(),
        None => String::from("unknown"),
      }
    }
    SongProperty::Length => {
      let (min, sec) = match song.duration {
        Some(d) => (d.num_minutes(), d.num_seconds() % 60),
        None => (0, 0),
      };
      return format!("{min}:{sec:02}", min = min, sec = sec);
    }
    SongProperty::Track => {
      let track = match song.tags.get("Track") {
        Some(t) => t.parse::<u32>().unwrap_or(0),
        None => 0,
      };
      return format!("{:>02}", track);
    }
    SongProperty::TrackFull => {
      let track = get_song_info(song, &SongProperty::Track);
      let total = "12";
      return format!("{}/{:>02}", track, total);
    }
    _ => {
      // Use tags as is
      let tag_s = format!("{}", tag);
      return match song.tags.get(tag_s.as_str()) {
        Some(t) => t.clone(),
        None => String::from("unknown"),
      };
    }
  }
}

fn get_song_time(status: &Status) -> (Duration, Duration) {
  status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)))
}

/// Get a song's (hopefully unique) id.
fn get_song_id(song: &mpd::Song) -> Option<SongId> {
  return song.place.as_ref().map(|p| SongId::from(p.id));
}

fn get_song_bitrate(status: &Status) -> u32 {
  status.bitrate.unwrap_or(0u32)
}

// TODO: update names once concat_idents can be used here for the function name
macro_rules! register_actions(
  ($($fun:ident), *) => (
    $(
      pub fn $fun(model: &mut Model)
      {
        model.$fun();
      }
    )*
  )
);

// Register actions for closures
register_actions!(
  execute_command,
  playlist_play,
  playlist_pause,
  playlist_stop,
  playlist_clear,
  playlist_delete_items,
  playlist_previous,
  playlist_next,
  play_selected,
  process_mouse,
  resize_windows,
  scroll_down,
  scroll_up,
  page_down,
  page_up,
  move_home,
  move_end,
  show_help,
  show_playlist,
  show_server_info,
  toggle_bitrate_visibility,
  toggle_random,
  toggle_repeat,
  volume_down,
  volume_up
);

macro_rules! actions_to_map(
    ($($fun:ident), *) => (
      {
        let mut action_map: BTreeMap<String, Action<'m>> = BTreeMap::new();
        $(
          let name: &str = stringify!($fun);
          let desc: &str = ACTION_DESCRIPTION.get(&name).unwrap_or(&"Missing description");
          action_map.insert(name.to_string(), Action::new(name, desc, $fun));
        )*
          action_map
      }
    )
  );

pub fn get_action_map<'m>() -> BTreeMap<String, Action<'m>> {
  let action_map = actions_to_map!(
    execute_command,
    playlist_play,
    playlist_pause,
    playlist_stop,
    playlist_clear,
    playlist_delete_items,
    playlist_previous,
    playlist_next,
    play_selected,
    process_mouse,
    resize_windows,
    scroll_down,
    scroll_up,
    page_down,
    page_up,
    move_home,
    move_end,
    show_help,
    show_playlist,
    show_server_info,
    toggle_bitrate_visibility,
    toggle_random,
    toggle_repeat,
    volume_down,
    volume_up
  );

  return action_map;
}

/// Structure containing the status of changes to the model's data.
#[derive(Default)]
pub struct DataChangeStatus {
  /// Whether the current playlist was modified.
  playlist: bool,
  /// Whether the general status was modified.
  status: bool,
  /// Whether the current song was modified.
  current_song: bool,
}

impl DataChangeStatus {
  pub fn new() -> DataChangeStatus {
    // Suppose that the data is uninitialized <=> has changed
    DataChangeStatus {
      playlist: true,
      status: true,
      current_song: true,
    }
  }
}

/// Song id.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct SongId(u32);

impl From<mpd::Id> for SongId {
  fn from(id: mpd::Id) -> Self {
    SongId(id.0)
  }
}

/// Structure containing the current MPD data.
struct Snapshot {
  /// Current MPD status.
  pub status: mpd::Status,
  /// Data relative to the current playlist.
  pub pl_info: PlaylistInfo,
  /// Queue (current playlist).
  pub queue: CachedValue<Vec<mpd::Song>>,
}

impl Snapshot {
  pub fn new() -> Snapshot {
    Snapshot {
      status: mpd::Status::default(),
      pl_info: PlaylistInfo::new(),
      queue: CachedValue::new(Vec::new(), Duration::milliseconds(500)),
    }
  }

  pub fn update(&mut self, client: &mut mpd::Client, change: &DataChangeStatus) {
    if change.playlist {
      self.queue.get_or(|| client.queue().unwrap());
      self.pl_info.size = (*self.queue).len() as u32;
      let sum = (*self.queue).iter().fold(0i64, |sum, val| {
        sum + val.duration.unwrap_or_else(|| Duration::seconds(0)).num_seconds()
      });
      self.pl_info.duration = Duration::seconds(sum);
    }

    if change.status {
      self.status = client.status().unwrap();
    }
  }
}

pub struct Model<'m> {
  /// MPD client.
  client: mpd::Client<TcpStream>,
  /// TUI view.
  view: &'m mut View,
  /// Initial configuration.
  config: &'m Config,
  /// Current state configuration.
  params: ParamConfig,
  /// Current active window.
  active_window: ActiveWindow,
  /// Index of the currently selected song (if any).
  selected_song: Option<TimedValue<u32>>,
  /// Snapshot of MPD data.
  snapshot: Snapshot,
  /// Temporary info message.
  info_msg: Option<TimedValue<String>>,
  /// Map action names to action functions.
  action_map: BTreeMap<String, Action<'m>>,
  /// Flags allowing to track changes to the model's data.
  change_status: DataChangeStatus,
  /// Cache of song infos, given their ids.
  song_info_map: HashMap<(SongId, SongProperty), String>,
}

impl<'m> Model<'m> {
  pub fn new(view: &'m mut View, config: &'m Config) -> Model<'m> {
    // Instantiate client.
    let res = start_client(config);
    if res.is_err() {
      println!("MPD not running. Exiting...");
      process::exit(2);
    }
    let mut client = res.unwrap();
    let mut snapshot = Snapshot::new();
    snapshot.queue.set(client.queue().unwrap());

    Model {
      client: client,
      view: view,
      config: config,
      params: config.params.clone(),
      active_window: ActiveWindow::Playlist,
      selected_song: None,
      snapshot: snapshot,
      info_msg: None,
      action_map: get_action_map(),
      change_status: DataChangeStatus::new(),
      song_info_map: HashMap::default(),
    }
  }

  pub fn playlist_play(&mut self) {
    if self.client.play().is_err() {
      self.update_message("Error: play failed");
    }

    self.change_status.current_song = true;
  }

  pub fn playlist_pause(&mut self) {
    let state = self.snapshot.status.state;

    match state {
      State::Play => {
        if self.client.pause(true).is_err() {
          self.update_message("Error: pause failed");
        }
      }
      State::Pause => {
        if self.client.pause(false).is_err() {
          self.update_message("Error: unpause failed");
        }
      }
      State::Stop => {
        // do nothing
      }
    }

    self.change_status.status = true;
  }

  pub fn read_input_command(&mut self) -> String {
    return self.view.read_input_command();
  }

  pub fn execute_command(&mut self) {
    let cmd = self.read_input_command();

    // Copy action to satisfy borrow checker
    let opt_action: Option<Action<'m>> = match self.action_map.get(cmd.as_str()) {
      Some(action) => Some(action.clone()),
      None => None,
    };

    match opt_action {
      Some(ref action) => {
        action.execute(self);
        self.update_message(format!("Executing command \"{}\"", cmd).as_str())
      }
      None => self.update_message(format!("No command named \"{}\"", cmd).as_str()),
    }

    self.change_status.playlist = true;
    self.change_status.status = true;
    self.change_status.current_song = true;
  }

  pub fn playlist_stop(&mut self) {
    if self.client.stop().is_err() {
      self.update_message("Error: stop failed");
    }

    self.change_status.status = true;
    self.change_status.current_song = true;
  }

  pub fn playlist_previous(&mut self) {
    if self.client.prev().is_err() {
      self.update_message("Error: previous song failed");
    }

    self.change_status.current_song = true;
  }

  pub fn playlist_next(&mut self) {
    if self.client.next().is_err() {
      self.update_message("Error: next song failed");
    }

    self.change_status.current_song = true;
  }

  pub fn playlist_clear(&mut self) {
    if self.client.clear().is_err() {
      self.update_message("Error: playlist clear failed");
    }

    self.change_status.playlist = true;
    self.change_status.current_song = true;
  }

  pub fn playlist_delete_items(&mut self) {
    if let Some(ref s) = self.selected_song {
      self.client.delete(s.value).unwrap_or(())
    };

    self.change_status.playlist = true;
  }

  pub fn play_selected(&mut self) {
    if let Some(ref s) = self.selected_song {
      self.client.switch(s.value).unwrap_or(())
    };

    self.change_status.status = true;
    self.change_status.current_song = true;
  }

  pub fn get_volume(&mut self) -> i8 {
    return self.snapshot.status.volume;
  }

  pub fn set_volume(&mut self, mut vol: i8) {
    // Volume ∈ [0,100]
    if vol < 0 {
      vol = 0;
    } else if vol > 100 {
      vol = 100;
    };
    if self.client.volume(vol).is_err() {
      self.update_message("Error: volume set failed");
    }

    self.change_status.status = true;
  }

  pub fn toggle_bitrate_visibility(&mut self) {
    self.params.display_bitrate = !self.params.display_bitrate;

    self.change_status.status = true;
  }

  pub fn toggle_random(&mut self) {
    let random = self.snapshot.status.random;
    if self.client.random(!random).is_err() {
      self.update_message("Error: random toggle failed");
    }

    self.change_status.status = true;
  }

  pub fn toggle_repeat(&mut self) {
    let repeat = self.snapshot.status.repeat;
    if self.client.repeat(!repeat).is_err() {
      self.update_message("Error: repeat toggle failed");
    }

    self.change_status.status = true;
  }

  pub fn set_song_progress(&mut self, pct: f32) {
    let (_, d) = get_song_time(&self.snapshot.status);
    let duration = d.num_seconds();
    let new_pos = Duration::seconds((duration as f32 * pct) as i64);
    let _res = self.client.rewind(new_pos);
  }

  pub fn process_mouse(&mut self) {
    let event = self.view.process_mouse();
    match event {
      MouseEvent::Nothing => {}
      MouseEvent::WakeUp => {
        if self.selected_song.is_some() {
          self.selected_song.as_mut().unwrap().bump();
        }
      }
      MouseEvent::ScrollDown => self.scroll_down(),
      MouseEvent::ScrollUp => self.scroll_up(),
      MouseEvent::SetProgress(pct) => self.set_song_progress(pct),
      MouseEvent::SetSelectedSong(idx) => self.selected_song = Some(TimedValue::<u32>::new(idx)),
    };
  }

  pub fn volume_up(&mut self) {
    let vol = self.get_volume();
    let step = self.config.params.volume_change_step;
    self.set_volume(vol + step);

    self.change_status.status = true;
  }

  pub fn volume_down(&mut self) {
    let vol = self.get_volume();
    let step = self.config.params.volume_change_step;
    self.set_volume(vol - step);

    self.change_status.status = true;
  }

  pub fn update_header(&mut self) {
    let vol: Option<i8> = if self.params.display_volume_level {
      Some(self.get_volume())
    } else {
      None
    };

    self.view.display_header(&self.active_window, &self.snapshot.pl_info, vol);
  }

  pub fn update_stateline(&mut self) {
    let status = &self.snapshot.status;

    let mut flags = Vec::<char>::new();
    if status.repeat {
      flags.push('r');
    }
    if status.random {
      flags.push('z');
    }
    if status.single {
      flags.push('s');
    }
    if status.consume {
      flags.push('c');
    }
    if status.crossfade.unwrap_or_else(|| Duration::seconds(0)).num_seconds() > 0 {
      flags.push('x');
    }
    self.view.display_stateline(&flags);
  }

  pub fn update_main_window(&mut self) {
    match self.active_window {
      ActiveWindow::Help => self.update_help(),
      ActiveWindow::Playlist => self.update_playlist(),
      ActiveWindow::ServerInfo => self.update_server_info(),
    }
  }

  pub fn update_help(&mut self) {
    self.view.display_help();
  }

  pub fn update_server_info(&mut self) {
    // Mutable getter for server stats
    self.view.display_server_info(&mut self.client);
  }

  pub fn fill_grid_data(&mut self, columns: &[Column]) -> Vec<String> {
    let n_cols = columns.len();
    let n_entries = (*self.snapshot.queue).len();
    let mut grid_raw = vec![String::new(); n_cols * n_entries];
    let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_mut(n_cols).collect();
    let grid: &mut [&mut [String]] = grid_base.as_mut_slice();
    // TODO: determine when to invalidate the cache
    let song_info_map = &mut self.song_info_map;

    // Fill data grid
    for (i, row) in grid.iter_mut().enumerate() {
      // Get song id
      let id = get_song_id(&(*self.snapshot.queue)[i]);

      for (j, cell) in row.iter_mut().enumerate() {
        let col_type = &columns[j].column_type;

        // Search for info in the map, or initialize it
        if let Some(id_value) = id {
          let info = song_info_map.entry((id_value, col_type.clone())).or_insert_with(String::new);
          if info.is_empty() {
            *info = get_song_info(&(*self.snapshot.queue)[i], &col_type);
          }
          *cell = info.clone();
        } else {
          // TODO: avoid code duplication
          *cell = get_song_info(&(*self.snapshot.queue)[i], &col_type);
        }
      }
    }

    return grid_raw;
  }

  pub fn update_playlist(&mut self) {
    // Get grid data and convert to 2D slices
    let columns = &self.config.params.song_columns_list_format;
    let n_cols = columns.len();
    // TODO: reuse data rather than reallocating for every call
    let grid_data = &mut self.fill_grid_data(&columns);
    let grid_base: Vec<_> = grid_data.as_slice().chunks(n_cols).collect();
    let grid: &[&[String]] = grid_base.as_slice();

    // Get index of current song
    let song = self.snapshot.status.song;
    let cur_song = if song.is_some() { Some(song.unwrap().pos) } else { None };

    self.view.display_main_playlist(&columns, &grid, cur_song, &self.selected_song);
  }

  pub fn update_progressbar(&mut self) {
    let (e, d) = get_song_time(&self.snapshot.status);
    let elapsed = e.num_seconds();
    let duration = d.num_seconds();

    let pct = if duration > 0 { (100 * elapsed / duration) as f32 } else { 0. };
    self.view.display_progressbar(pct);
  }

  pub fn update_statusbar(&mut self) {
    use mpd::status::State;

    // If an info message has to be displayed
    if self.info_msg.is_some() {
      if get_time() < self.info_msg.as_ref().unwrap().timestamp + Duration::seconds(5) {
        self.view.display_statusbar_msg(&self.info_msg.as_ref().unwrap().value);
        return;
      } else {
        self.info_msg = None;
      }
    }

    let mut mode = String::default();
    let mut msg = String::default();
    let mut track = String::default();

    let query = self.client.currentsong();
    if query.is_ok() {
      let data = query.unwrap();
      if data.is_some() {
        let status = &self.snapshot.status;
        let state = status.state;
        match state {
          State::Play => {
            mode = "Playing".to_string();
          }
          State::Pause => {
            mode = "Paused".to_string();
          }
          State::Stop => {
            mode = "Stopped".to_string();
          }
        }

        let song = data.unwrap();
        let artist = get_song_info(&song, &SongProperty::Artist);
        let album = get_song_info(&song, &SongProperty::Album);
        let title = song.title.unwrap_or_else(|| "Unknown title".to_string());
        msg = format!("{} - {} - {}", artist, title, album);

        let mut bitrate = String::default();
        let (cur, total) = get_song_time(&status);
        let cur_min = cur.num_minutes();
        let cur_sec = cur.num_seconds() % 60;
        let total_min = total.num_minutes();
        let total_sec = total.num_seconds() % 60;
        if self.params.display_bitrate {
          let val = get_song_bitrate(&status);
          if val > 0 {
            bitrate = format!("({} kbps) ", val);
          }
        }
        track = format!("{}[{}:{:02}/{}:{:02}]", bitrate, cur_min, cur_sec, total_min, total_sec);
      }
    } else {
      mode = "No MPD status available".to_string();
    }
    self.view.display_statusbar(&mode, &msg, &track);
  }

  pub fn update_message(&mut self, msg: &str) {
    self.info_msg = Some(TimedValue::<String>::new(String::from(msg)));
    self.change_status.status = true;
  }

  pub fn resize_windows(&mut self) {
    self.view.resize_windows();
  }

  pub fn scroll_playlist(&mut self, offset: i32) {
    let end = self.snapshot.pl_info.size as i32;
    self.selected_song = Some(TimedValue::<u32>::new(match self.selected_song {
      Some(ref s) => {
        let cur_pos = s.value as i32;
        // If we're going past the end of the playlist
        if cur_pos >= end - offset {
          if self.params.cyclic_scrolling {
            (cur_pos + offset - end) as u32
          } else {
            end as u32
          }
        // If we're going past the start of the playlist
        } else if cur_pos < -offset {
          if self.params.cyclic_scrolling {
            (end + cur_pos + offset) as u32
          } else {
            0
          }
        // We're staying within the playlist
        } else {
          (cur_pos + offset) as u32
        }
      }
      None => 0,
    }))
  }

  pub fn scroll_help(&mut self, offset: i32) {
    self.view.help.scroll(offset);
  }

  pub fn scroll_down(&mut self) {
    match self.active_window {
      ActiveWindow::Help => self.scroll_help(1),
      ActiveWindow::Playlist => self.scroll_playlist(1),
      _ => {}
    }
  }

  pub fn page_down(&mut self) {
    let offset = self.view.get_main_playlist_height();
    match self.active_window {
      // TODO: use adaptative height
      ActiveWindow::Help => self.scroll_help(offset),
      ActiveWindow::Playlist => self.scroll_playlist(offset),
      _ => {}
    }
  }

  pub fn scroll_up(&mut self) {
    match self.active_window {
      ActiveWindow::Help => self.scroll_help(-1),
      ActiveWindow::Playlist => self.scroll_playlist(-1),
      _ => {}
    }
  }

  pub fn page_up(&mut self) {
    let offset = -self.view.get_main_playlist_height();
    match self.active_window {
      // TODO: use adaptative height
      ActiveWindow::Help => self.scroll_help(offset),
      ActiveWindow::Playlist => self.scroll_playlist(offset),
      _ => {}
    }
  }

  pub fn move_home(&mut self) {
    self.selected_song = Some(TimedValue::<u32>::new(0));
  }

  pub fn move_end(&mut self) {
    let end = self.snapshot.pl_info.size;
    self.selected_song = Some(TimedValue::<u32>::new(end - 1));
  }

  pub fn show_help(&mut self) {
    self.active_window = ActiveWindow::Help;
  }

  pub fn show_server_info(&mut self) {
    self.active_window = ActiveWindow::ServerInfo;
  }

  pub fn show_playlist(&mut self) {
    self.active_window = ActiveWindow::Playlist;
  }

  pub fn take_snapshot(&mut self) {
    self.snapshot.update(&mut self.client, &self.change_status);
    // TODO: reset
    self.change_status = DataChangeStatus::new();
  }
}
