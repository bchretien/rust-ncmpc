extern crate ncurses;
extern crate ini;
extern crate xdg;

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use std::path::PathBuf;

use ini::Ini;
use ncurses as nc;

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum ControlKey {
  KeyCode(i32),
  Char(char),
}

pub type ControlKeys = Vec<ControlKey>;

/// Key bindings configuration.
#[derive(Clone,PartialEq,Debug)]
pub struct KeyConfig {
  pub clear: ControlKeys,
  pub delete: ControlKeys,
  pub next_song: ControlKeys,
  pub play_pause: ControlKeys,
  pub press_enter: ControlKeys,
  pub previous_song: ControlKeys,
  pub quit: ControlKeys,
  pub scroll_down: ControlKeys,
  pub scroll_up: ControlKeys,
  pub stop: ControlKeys,
  pub toggle_bitrate_visibility: ControlKeys,
  pub toggle_random: ControlKeys,
  pub toggle_repeat: ControlKeys,
  pub volume_down: ControlKeys,
  pub volume_up: ControlKeys,
}

#[derive(Clone,Copy,PartialEq,Debug)]
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

#[derive(Clone,PartialEq,Debug)]
pub struct ParamConfig {
  pub cyclic_scrolling: bool,
  pub display_bitrate: bool,
  pub display_remaining_time: bool,
  pub display_volume_level: bool,
  pub header_text_scrolling: bool,
  pub mpd_host: String,
  pub mpd_port: u16,
  pub volume_change_step: i8,
}

#[derive(Clone,PartialEq,Debug)]
pub struct Config {
  pub colors: ColorConfig,
  pub keys: KeyConfig,
  pub params: ParamConfig,
}

pub struct ConfigLoader {
  default_config_path: Option<PathBuf>,
  default_bindings_path: Option<PathBuf>,
}

impl KeyConfig {
  pub fn new() -> KeyConfig {
    KeyConfig {
      clear: vec![ControlKey::Char('c')],
      delete: vec![ControlKey::KeyCode(nc::KEY_DC)],
      next_song: vec![ControlKey::Char('>')],
      play_pause: vec![ControlKey::Char('p')],
      press_enter: vec![ControlKey::Char('\n')],
      previous_song: vec![ControlKey::Char('<')],
      quit: vec![ControlKey::Char('q')],
      scroll_down: vec![ControlKey::KeyCode(nc::KEY_DOWN)],
      scroll_up: vec![ControlKey::KeyCode(nc::KEY_UP)],
      stop: vec![ControlKey::Char('s')],
      toggle_bitrate_visibility: vec![ControlKey::Char('#')],
      toggle_random: vec![ControlKey::Char('z')],
      toggle_repeat: vec![ControlKey::Char('r')],
      volume_down: vec![ControlKey::KeyCode(nc::KEY_LEFT)],
      volume_up: vec![ControlKey::KeyCode(nc::KEY_RIGHT)],
    }
  }
}

pub trait ToKeyCode {
  fn keycode(&self) -> i32;
}

impl ToKeyCode for i32 {
  fn keycode(&self) -> i32 {
    *self
  }
}

impl ToKeyCode for char {
  fn keycode(&self) -> i32 {
    *self as i32
  }
}

impl ToKeyCode for ControlKey {
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

impl ParamConfig {
  pub fn new() -> ParamConfig {
    ParamConfig {
      cyclic_scrolling: false,
      display_bitrate: false,
      display_remaining_time: false,
      display_volume_level: true,
      header_text_scrolling: true,
      mpd_host: String::from("localhost"),
      mpd_port: 6600,
      volume_change_step: 2,
    }
  }
}

impl Config {
  pub fn new() -> Config {
    let keys = KeyConfig::new();
    let mut params = ParamConfig::new();

    // Search for the MPD environment variables, as they take precedence over
    // the configuration
    let mpd_host = env::var("MPD_HOST");
    let mpd_port = env::var("MPD_PORT");
    if mpd_host.is_ok() {
      params.mpd_host = mpd_host.unwrap();
    }
    if mpd_port.is_ok() {
      params.mpd_port = mpd_port.unwrap()
        .parse::<u16>()
        .unwrap_or(params.mpd_port);
    }

    Config {
      colors: ColorConfig::new(),
      keys: keys,
      params: params,
    }
  }

  /// Get the socket address of the MPC daemon.
  pub fn socket_addr(&self) -> SocketAddr {
    let ip = if self.params.mpd_host == "localhost" {
      IpAddr::from_str("127.0.0.1").unwrap()
    } else {
      IpAddr::from_str(self.params.mpd_host.as_str()).unwrap()
    };
    (ip, self.params.mpd_port).to_socket_addrs().unwrap().next().unwrap()
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

fn parse_bool(s: &str) -> bool {
  s == "yes"
}

fn parse_int<T>(s: &str) -> T
  where T: FromStr,
        <T as FromStr>::Err: Debug
{
  let res = s.parse::<T>();
  if res.is_err() {
    panic!(format!("Error while parsing \"{}\" as an integer", s));
  }
  res.unwrap()
}

fn assign(key: &str, val: &str, config: &mut Config) -> bool {
  match key {
    // Colors
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
    // Parameters
    "cyclic_scrolling" => config.params.cyclic_scrolling = parse_bool(val),
    "display_bitrate" => config.params.display_bitrate = parse_bool(val),
    "display_remaining_time" => config.params.display_remaining_time = parse_bool(val),
    "display_volume_level" => config.params.display_volume_level = parse_bool(val),
    "header_text_scrolling" => config.params.header_text_scrolling = parse_bool(val),
    "mpd_host" => config.params.mpd_host = String::from(val),
    "mpd_port" => config.params.mpd_port = parse_int(val),
    "volume_change_step" => config.params.volume_change_step = parse_int(val),
    _ => return false,
  }
  return true;
}

impl ConfigLoader {
  pub fn new() -> ConfigLoader {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("ncmpcpp").unwrap();

    let default_config_path = xdg_dirs.find_config_file("config");
    let default_bindings_path = xdg_dirs.find_config_file("bindings");

    ConfigLoader {
      default_config_path: default_config_path,
      default_bindings_path: default_bindings_path,
    }
  }

  pub fn load(&self, user_config: Option<PathBuf>) -> Config {
    let opt_path = if user_config.is_some() { user_config.clone() } else { self.default_config_path.clone() };

    let mut config = Config::new();

    // Read ncmpcpp configuration
    if opt_path.is_some() {
      let path = opt_path.unwrap();
      let file = path.to_str().unwrap();
      let ini = Ini::load_from_file(file).unwrap();
      for (_, prop) in ini.iter() {
        for (k, v) in prop.iter() {
          // Remove quotes
          let fixed = v.trim_matches('\"');
          assign(&k, fixed, &mut config);
        }
      }
    }

    return config;
  }
}
