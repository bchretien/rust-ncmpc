extern crate time;
extern crate mpd;

use std::process;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use time::Duration;

use view::*;
use config::*;
use mpd::status::State;
use mpd::status::Status;
use mpd::song::Song;

pub type SharedModel<'m> = Arc<Mutex<Model<'m>>>;

fn start_client(config: &Config) -> Result<mpd::Client, mpd::error::Error> {
  mpd::Client::connect(config.addr)
}

fn get_song_info(song: &Song, tag: &String) -> String {
  let unknown = "unknown".to_string();
  let zero = "0".to_string();
  if tag == "Title" {
    return song.clone().title.unwrap_or(unknown);
  } else if tag == "Time" || tag == "Duration" {
    let dur = song.clone().duration.unwrap_or(Duration::seconds(0));
    let min = dur.num_minutes();
    let sec = dur.num_seconds() % 60;
    return format!("{min}:{sec:02}", min = min, sec = sec);
  } else if tag == "Track" {
    let track = song.tags.get(tag).unwrap_or(&zero).to_string();
    let track_s = track.parse::<u32>().unwrap_or(0);
    return format!("{:>02}", track_s);
  } else
  // Use tags as is
  {
    return song.tags.get(tag).unwrap_or(&unknown).to_string();
  }
}

fn get_song_time(status: &Status) -> (Duration, Duration) {
  status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)))
}

fn get_song_bitrate(status: &Status) -> u32 {
  status.bitrate.unwrap_or(0u32)
}

// TODO: update names once concat_idents can be used here for the function name
macro_rules! register_action(
    ($model_fun:ident) => (
        pub fn $model_fun(shared_model: &mut SharedModel)
        {
            let mut model = shared_model.lock().unwrap();
            model.$model_fun();
        }
    )
);

// Register actions for closures
register_action!(playlist_play);
register_action!(playlist_pause);
register_action!(playlist_stop);
register_action!(playlist_clear);
register_action!(playlist_previous);
register_action!(playlist_next);
register_action!(play_selected);
register_action!(process_mouse);
register_action!(resize_windows);
register_action!(toggle_bitrate_visibility);
register_action!(toggle_random);
register_action!(toggle_repeat);
register_action!(volume_down);
register_action!(volume_up);

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
  /// Currently selected song (if any).
  selected_song: Option<u32>,
  /// Snapshot of MPD data.
  snapshot: Snapshot,
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
    }
  }

  pub fn playlist_play(&mut self) {
    if self.client.play().is_ok() {
      self.view.display_debug_prompt("Playing");
    }
  }

  pub fn playlist_pause(&mut self) {
    let state = self.snapshot.status.state;

    match state {
      State::Play => {
        if self.client.pause(true).is_ok() {
          self.view.display_debug_prompt("Pausing");
        }
      }
      State::Pause => {
        if self.client.pause(false).is_ok() {
          self.view.display_debug_prompt("Playing");
        }
      }
      State::Stop => {
        // do nothing
      }
    }
  }

  pub fn playlist_stop(&mut self) {
    if self.client.stop().is_ok() {
      self.view.display_debug_prompt("Stopping");
    }
  }

  pub fn playlist_previous(&mut self) {
    if self.client.prev().is_ok() {
      self.view.display_debug_prompt("Previous song");
    }
  }

  pub fn playlist_next(&mut self) {
    if self.client.next().is_ok() {
      self.view.display_debug_prompt("Next song");
    }
  }

  pub fn playlist_clear(&mut self) {
    if self.client.clear().is_ok() {
      self.view.display_debug_prompt("Cleared playlist");
    }
  }

  pub fn play_selected(&mut self) {
    match self.selected_song {
      Some(idx) => self.client.switch(idx).unwrap_or({}),
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
    if self.client.volume(vol).is_ok() {
      self.view.display_debug_prompt("Volume set");
    }
  }

  pub fn toggle_bitrate_visibility(&mut self) {
    self.params.display_bitrate = !self.params.display_bitrate;
  }

  pub fn toggle_random(&mut self) {
    let random = self.snapshot.status.random;
    if self.client.random(!random).is_err() {
      self.view.display_debug_prompt("Failed to toggle random");
    }
  }

  pub fn toggle_repeat(&mut self) {
    let repeat = self.snapshot.status.repeat;
    if self.client.repeat(!repeat).is_err() {
      self.view.display_debug_prompt("Failed to toggle repeat");
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
      MouseEvent::SetProgress(pct) => self.set_song_progress(pct),
      MouseEvent::SetSelectedSong(idx) => self.selected_song = Some(idx),
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
    self.view.display_debug_prompt(msg);
  }

  pub fn resize_windows(&mut self) {
    self.view.resize_windows();
  }

  pub fn take_snapshot(&mut self) {
    self.snapshot.update(&mut self.client);
  }
}
