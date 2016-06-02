extern crate ncurses;
extern crate ini;

use std::net::SocketAddr;
use std::env;
use std::path::{Path, PathBuf};

use ini::Ini;
use ncurses as nc;

#[derive(Clone,Copy)]
pub enum ControlKey {
  KeyCode(i32),
  Char(char),
}

#[derive(Clone,Copy)]
pub struct KeyConfig {
  pub clear: ControlKey,
  pub next_song: ControlKey,
  pub play_pause: ControlKey,
  pub previous_song: ControlKey,
  pub quit: ControlKey,
  pub stop: ControlKey,
}

#[derive(Clone,Copy)]
pub struct ColorConfig {
  pub color1: i16,
  pub color2: i16,
  pub header_window: i16,
  pub main_window: i16,
  pub main_window_highlight: i16,
  pub progressbar: i16,
  pub progressbar_elapsed: i16,
  pub state_flags: i16,
  pub state_line: i16,
  pub statusbar: i16,
  pub volume: i16,
}

#[derive(Clone,Copy)]
pub struct Config {
  pub addr: SocketAddr,
  pub colors: ColorConfig,
  pub keys: KeyConfig,
}

pub struct ConfigLoader {
  default_config_path: PathBuf,
}

impl KeyConfig {
  pub fn new() -> KeyConfig {
    KeyConfig {
      clear: ControlKey::Char('c'),
      next_song: ControlKey::Char('>'),
      play_pause: ControlKey::Char('p'),
      previous_song: ControlKey::Char('<'),
      quit: ControlKey::Char('q'),
      stop: ControlKey::Char('s'),
    }
  }
}

pub trait toKeyCode {
  fn keycode(&self) -> i32;
}

impl toKeyCode for i32 {
  fn keycode(&self) -> i32 {
    *self
  }
}

impl toKeyCode for char {
  fn keycode(&self) -> i32 {
    *self as i32
  }
}

impl toKeyCode for ControlKey {
  fn keycode(&self) -> i32 {
    match *self {
      ControlKey::KeyCode(c) => return c,
      ControlKey::Char(c) => return c.keycode(),
    }
  }
}

impl ColorConfig {
  pub fn new() -> ColorConfig {
    ColorConfig {
      color1: 0,
      color2: 0,
      header_window: 0,
      main_window: 0,
      main_window_highlight: 0,
      progressbar: 0,
      progressbar_elapsed: 0,
      state_flags: 0,
      state_line: 0,
      statusbar: 0,
      volume: 0,
    }
  }
}

impl Config {
  pub fn new() -> Config {
    // TODO: support MPD_SOCK
    let addr = env::var("MPD_SOCK").unwrap_or("127.0.0.1:6600".to_owned());

    // Search for the MPD_PORT environment variable
    let mpd_ip = "127.0.0.1".parse().unwrap();
    let mpd_port = env::var("MPD_PORT")
      .unwrap_or("6600".to_owned())
      .parse::<u16>()
      .unwrap_or(6600);
    println!("MPD: {}:{}", mpd_ip, mpd_port);

    let keys = KeyConfig::new();

    Config {
      colors: ColorConfig::new(),
      addr: SocketAddr::new(mpd_ip, mpd_port),
      keys: keys,
    }
  }
}

fn parse_color(s: &str) -> i16 {
  match s {
    "black" => nc::COLOR_BLACK,
    "red" => nc::COLOR_RED,
    "green" => nc::COLOR_GREEN,
    "yellow" => nc::COLOR_YELLOW,
    "blue" => nc::COLOR_BLUE,
    "magenta" => nc::COLOR_MAGENTA,
    "cyan" => nc::COLOR_CYAN,
    "white" => nc::COLOR_WHITE,
    _ => -1,
  }
}

fn assign(key: &str, val: &str, config: &mut Config) -> bool {
  match key {
    "color1" => config.colors.color1 = parse_color(val),
    "color2" => config.colors.color2 = parse_color(val),
    "header_window_color" => config.colors.header_window = parse_color(val),
    "progressbar_color" => config.colors.progressbar = parse_color(val),
    "progressbar_elapsed_color" => config.colors.progressbar_elapsed = parse_color(val),
    "main_window_color" => config.colors.main_window = parse_color(val),
    "main_window_highlight_color" => config.colors.main_window_highlight = parse_color(val),
    "state_flags_color" => config.colors.state_flags = parse_color(val),
    "state_line_color" => config.colors.state_line = parse_color(val),
    "statusbar_color" => config.colors.statusbar = parse_color(val),
    "volume_color" => config.colors.volume = parse_color(val),
    _ => return false,
  }
  return true;
}

impl ConfigLoader {
  pub fn new() -> ConfigLoader {
    // TODO: rely on XDG paths
    let mut default_config_path = PathBuf::from("");
    match env::home_dir() {
      Some(path) => default_config_path = path.join(PathBuf::from(".config/ncmpcpp/config")),
      None => default_config_path = PathBuf::from(""),
    }

    ConfigLoader { default_config_path: default_config_path }
  }

  pub fn load(&self, opt_path: Option<PathBuf>) -> Config {
    let mut path: PathBuf = PathBuf::from("");
    match opt_path {
      Some(x) => path = x,
      None => path = self.default_config_path.clone(),
    }

    let mut config = Config::new();

    // Read ncmpcpp configuration
    let i = Ini::load_from_file(path.to_str().unwrap()).unwrap();
    for (sec, prop) in i.iter() {
      for (k, v) in prop.iter() {
        // Remove quotes
        let fixed = v.trim_matches('\"');
        assign(&k, fixed, &mut config);
      }
    }

    return config;
  }
}
