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
    return format!("{min}:{sec:>02}", min = min, sec = sec);
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
register_action!(volume_down);
register_action!(volume_up);

pub struct Model<'m> {
  /// MPD client.
  client: mpd::Client<TcpStream>,
  /// TUI view.
  view: &'m mut View,
  /// Configuration.
  config: &'m Config,
  /// Data relative to the current playlist.
  pl_data: PlaylistData,
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

    Model {
      client: client,
      view: view,
      config: config,
      pl_data: PlaylistData::new(),
    }
  }

  pub fn playlist_play(&mut self) {
    if self.client.play().is_ok() {
      self.view.display_debug_prompt("Playing");
    }
  }

  pub fn playlist_pause(&mut self) {
    let status = self.client.status();
    if status.is_err() {
      self.view.display_debug_prompt(&format!("{}", status.unwrap_err()));
      return;
    }
    let state = status.unwrap().state;

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

  pub fn get_volume(&mut self) -> i8 {
    let status = self.client.status();
    if status.is_err() {
      self.view.display_debug_prompt(&format!("{}", status.unwrap_err()));
      return 0;
    }
    return status.unwrap().volume;
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
    self.pl_data.size = queue.len() as u32;
    let sum = queue.iter().fold(0i64,
                                |sum, val| sum + val.duration.unwrap_or(Duration::seconds(0)).num_seconds());
    self.pl_data.duration = Duration::seconds(sum);
  }

  pub fn update_header(&mut self) {
    let vol = self.get_volume();
    // TODO: select when to reload data
    if self.pl_data.size == 0 {
      self.reload_playlist_data();
    }
    self.view.display_header(&self.pl_data, vol);
  }

  pub fn update_stateline(&mut self) {
    let query = self.client.status();
    if query.is_err() {
      self.view.display_debug_prompt(&format!("{}", query.unwrap_err()));
      return;
    }
    let status = query.unwrap();

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
    let song = self.client.status().unwrap().song;
    let mut pos: Option<u32> = None;
    if song.is_some() {
      pos = Some(song.unwrap().pos);
    }

    self.view.display_main_playlist(&columns, grid, pos);
  }

  pub fn update_progressbar(&mut self) {
    let (e, d) = get_song_time(&self.client.status().unwrap());
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

    let query = self.client.currentsong();
    if query.is_ok() {
      let data = query.unwrap();
      if data.is_some() {
        let state = self.client.status().unwrap().state;
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
      }
    } else {
      mode = "No MPD status available".to_string();
    }
    self.view.display_statusbar(&mode, &msg);
  }

  pub fn update_message(&mut self, msg: &str) {
    self.view.display_debug_prompt(msg);
  }
}
