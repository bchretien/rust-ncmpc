extern crate time;
extern crate mpd;

use std::process;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use time::{Duration, get_time};

use view::*;
use config::*;
use util::TimedValue;
use mpd::status::{State, Status};
use mpd::song::Song;

pub type SharedModel<'m> = Arc<Mutex<Model<'m>>>;
pub type ActionCallback<'m> = fn(&mut SharedModel<'m>);

/// Action triggered by the user.
pub struct Action<'m> {
  callback: ActionCallback<'m>,
}

impl<'m> Action<'m> {
  pub fn new(func: ActionCallback<'m>) -> Action<'m> {
    Action { callback: func }
  }

  pub fn execute(&self, model: &mut SharedModel<'m>) {
    (self.callback)(model);
  }
}

impl<'m> Clone for Action<'m> {
  fn clone(&self) -> Action<'m> {
    let callback: ActionCallback<'m> = self.callback;
    return Action { callback: callback };
  }
}

fn start_client(config: &Config) -> Result<mpd::Client, mpd::error::Error> {
  mpd::Client::connect(config.socket_addr())
}

fn get_song_info(song: &Song, tag: &String) -> String {
  if tag == "Title" {
    return match song.title.as_ref() {
      Some(t) => t.clone(),
      None => String::from("unknown"),
    };
  } else if tag == "Time" || tag == "Duration" {
    let (min, sec) = match song.duration {
      Some(d) => (d.num_minutes(), d.num_seconds() % 60),
      None => (0, 0),
    };
    return format!("{min}:{sec:02}", min = min, sec = sec);
  } else if tag == "Track" {
    let track = match song.tags.get(tag) {
      Some(t) => t.parse::<u32>().unwrap_or(0),
      None => 0,
    };
    return format!("{:>02}", track);
  } else
  // Use tags as is
  {
    return match song.tags.get(tag) {
      Some(t) => t.clone(),
      None => String::from("unknown"),
    };
  }
}

fn get_song_time(status: &Status) -> (Duration, Duration) {
  status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)))
}

fn get_song_bitrate(status: &Status) -> u32 {
  status.bitrate.unwrap_or(0u32)
}

// TODO: update names once concat_idents can be used here for the function name
macro_rules! register_actions(
  ($($fun:ident), *) => (
    $(
      pub fn $fun(shared_model: &mut SharedModel)
      {
        let mut model = shared_model.lock().unwrap();
        model.$fun();
      }
    )*
  )
);

// Register actions for closures
register_actions!(
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
  toggle_bitrate_visibility,
  toggle_random,
  toggle_repeat,
  volume_down,
  volume_up);

macro_rules! actions_to_map(
  ($($fun:ident), *) => (
    {
      let mut action_map: HashMap<String, Action<'m>> = HashMap::new();
      $(
        action_map.insert(stringify!($fun).to_string(), Action::new($fun));
      )*
      action_map
    }
  )
);

pub fn get_action_map<'m>() -> HashMap<String, Action<'m>> {
  let action_map = actions_to_map!(
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
    toggle_bitrate_visibility,
    toggle_random,
    toggle_repeat,
    volume_down,
    volume_up);

  return action_map;
}

struct Snapshot {
  /// Current MPD status.
  pub status: mpd::Status,
  /// Data relative to the current playlist.
  pub pl_data: PlaylistData,
}

impl Snapshot {
  pub fn new() -> Snapshot {
    Snapshot {
      status: mpd::Status::default(),
      pl_data: PlaylistData::new(),
    }
  }

  pub fn update(&mut self, client: &mut mpd::Client) {
    self.status = client.status().unwrap().clone();
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
  /// Index of the currently selected song (if any).
  selected_song: Option<TimedValue<u32>>,
  /// Snapshot of MPD data.
  snapshot: Snapshot,
  /// Temporary info message.
  info_msg: Option<TimedValue<String>>,
}

impl<'m> Model<'m> {
  pub fn new(view: &'m mut View, config: &'m Config) -> Model<'m> {
    // Instantiate client.
    let res = start_client(config);
    if res.is_err() {
      println!("MPD not running. Exiting...");
      process::exit(2);
    }
    let client = res.unwrap();

    let snapshot = Snapshot::new();

    Model {
      client: client,
      view: view,
      config: config,
      params: config.params.clone(),
      selected_song: None,
      snapshot: snapshot,
      info_msg: None,
    }
  }

  pub fn playlist_play(&mut self) {
    if self.client.play().is_err() {
      self.update_message("Error: play failed");
    }
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
  }

  pub fn playlist_stop(&mut self) {
    if self.client.stop().is_err() {
      self.update_message("Error: stop failed");
    }
  }

  pub fn playlist_previous(&mut self) {
    if self.client.prev().is_err() {
      self.update_message("Error: previous song failed");
    }
  }

  pub fn playlist_next(&mut self) {
    if self.client.next().is_err() {
      self.update_message("Error: next song failed");
    }
  }

  pub fn playlist_clear(&mut self) {
    if self.client.clear().is_err() {
      self.update_message("Error: playlist clear failed");
    }
  }

  pub fn playlist_delete_items(&mut self) {
    match self.selected_song {
      Some(ref s) => self.client.delete(s.value).unwrap_or({}),
      None => {}
    }
  }

  pub fn play_selected(&mut self) {
    match self.selected_song {
      Some(ref s) => self.client.switch(s.value).unwrap_or({}),
      None => {}
    };
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
  }

  pub fn toggle_bitrate_visibility(&mut self) {
    self.params.display_bitrate = !self.params.display_bitrate;
  }

  pub fn toggle_random(&mut self) {
    let random = self.snapshot.status.random;
    if self.client.random(!random).is_err() {
      self.update_message("Error: random toggle failed");
    }
  }

  pub fn toggle_repeat(&mut self) {
    let repeat = self.snapshot.status.repeat;
    if self.client.repeat(!repeat).is_err() {
      self.update_message("Error: repeat toggle failed");
    }
  }

  pub fn set_song_progress(&mut self, pct: f32) {
    let (_, d) = get_song_time(&self.snapshot.status);
    let duration = d.num_seconds();
    let new_pos = Duration::seconds((duration as f32 * pct) as i64);
    self.client.rewind(new_pos);
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
      MouseEvent::SetProgress(pct) => self.set_song_progress(pct),
      MouseEvent::SetSelectedSong(idx) => self.selected_song = Some(TimedValue::<u32>::new(idx)),
    };
  }

  pub fn volume_up(&mut self) {
    let vol = self.get_volume();
    let step = self.config.params.volume_change_step;
    self.set_volume(vol + step);
  }

  pub fn volume_down(&mut self) {
    let vol = self.get_volume();
    let step = self.config.params.volume_change_step;
    self.set_volume(vol - step);
  }

  fn reload_playlist_data(&mut self) {
    let queue = self.client.queue().unwrap_or(Vec::<Song>::default());
    self.snapshot.pl_data.size = queue.len() as u32;
    let sum = queue.iter().fold(0i64,
                                |sum, val| sum + val.duration.unwrap_or(Duration::seconds(0)).num_seconds());
    self.snapshot.pl_data.duration = Duration::seconds(sum);
  }

  pub fn update_header(&mut self) {
    let vol: Option<i8> = if self.params.display_volume_level { Some(self.get_volume()) } else { None };

    // TODO: select when to reload data
    if self.snapshot.pl_data.size == 0 {
      self.reload_playlist_data();
    }
    self.view.display_header(&self.snapshot.pl_data, vol);
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
    if status.crossfade.unwrap_or(Duration::seconds(0)).num_seconds() > 0 {
      flags.push('x');
    }
    self.view.display_stateline(&flags);
  }

  pub fn update_playlist(&mut self) {
    // Get playlist
    let playlist = self.client.queue().unwrap();

    let columns =
      [("Artist".to_string(), 20), ("Track".to_string(), 2), ("Title".to_string(), 40), ("Album".to_string(), 40), ("Time".to_string(), 5)];
    let n_cols = columns.len();
    let n_entries = playlist.len();
    let mut grid_raw = vec![String::from("a"); n_cols * n_entries];
    let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_mut(n_cols).collect();
    let mut grid: &mut [&mut [String]] = grid_base.as_mut_slice();

    for i in 0..n_entries {
      grid[i][0] = get_song_info(&playlist[i], &"Artist".to_string());
      grid[i][1] = get_song_info(&playlist[i], &"Track".to_string());
      grid[i][2] = get_song_info(&playlist[i], &"Title".to_string());
      grid[i][3] = get_song_info(&playlist[i], &"Album".to_string());
      grid[i][4] = get_song_info(&playlist[i], &"Time".to_string());
    }

    // Get index of current song
    let song = self.snapshot.status.song;
    let mut cur_song: Option<u32> = None;
    if song.is_some() {
      cur_song = Some(song.unwrap().pos);
    }

    self.view.display_main_playlist(&columns, grid, &cur_song, &self.selected_song);
  }

  pub fn update_progressbar(&mut self) {
    let (e, d) = get_song_time(&self.snapshot.status);
    let elapsed = e.num_seconds();
    let duration = d.num_seconds();

    let mut pct: f32 = 0.;
    if duration > 0 {
      pct = (100 * elapsed / duration) as f32;
    }
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
        let artist = get_song_info(&song, &"Artist".to_string());
        let album = get_song_info(&song, &"Album".to_string());
        let title = song.title.unwrap_or("Unknown title".to_string());
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
  }

  pub fn resize_windows(&mut self) {
    self.view.resize_windows();
  }

  pub fn scroll_down(&mut self) {
    let end = self.snapshot.pl_data.size;
    self.selected_song = Some(TimedValue::<u32>::new(match self.selected_song {
      Some(ref s) => if s.value == end - 1 { if self.params.cyclic_scrolling { 0 } else { s.value } } else { s.value + 1 },
      None => 0,
    }))
  }

  pub fn scroll_up(&mut self) {
    let end = self.snapshot.pl_data.size;
    self.selected_song = Some(TimedValue::<u32>::new(match self.selected_song {
      Some(ref s) => if s.value == 0 { if self.params.cyclic_scrolling { end - 1 } else { 0 } } else { s.value - 1 },
      None => 0,
    }))
  }

  pub fn take_snapshot(&mut self) {
    self.snapshot.update(&mut self.client);
  }
}
