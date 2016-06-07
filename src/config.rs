extern crate ncurses;
extern crate ini;

use std::net::SocketAddr;
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use std::path::PathBuf;

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
  pub delete: ControlKey,
  pub next_song: ControlKey,
  pub play_pause: ControlKey,
  pub press_enter: ControlKey,
  pub previous_song: ControlKey,
  pub quit: ControlKey,
  pub scroll_down: ControlKey,
  pub scroll_up: ControlKey,
  pub stop: ControlKey,
  pub toggle_bitrate_visibility: ControlKey,
  pub toggle_random: ControlKey,
  pub toggle_repeat: ControlKey,
  pub volume_down: ControlKey,
  pub volume_up: ControlKey,
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
pub struct ParamConfig {
  pub cyclic_scrolling: bool,
  pub display_bitrate: bool,
  pub display_remaining_time: bool,
  pub display_volume_level: bool,
  pub header_text_scrolling: bool,
  pub volume_change_step: i8,
}

#[derive(Clone,Copy)]
pub struct Config {
  pub addr: SocketAddr,
  pub colors: ColorConfig,
  pub keys: KeyConfig,
  pub params: ParamConfig,
}

pub struct ConfigLoader {
  default_config_path: PathBuf,
}

impl KeyConfig {
  pub fn new() -> KeyConfig {
    KeyConfig {
      clear: ControlKey::Char('c'),
      delete: ControlKey::KeyCode(nc::KEY_DC),
      next_song: ControlKey::Char('>'),
      play_pause: ControlKey::Char('p'),
      press_enter: ControlKey::Char('\n'),
      previous_song: ControlKey::Char('<'),
      quit: ControlKey::Char('q'),
      scroll_down: ControlKey::KeyCode(nc::KEY_DOWN),
      scroll_up: ControlKey::KeyCode(nc::KEY_UP),
      stop: ControlKey::Char('s'),
      toggle_bitrate_visibility: ControlKey::Char('#'),
      toggle_random: ControlKey::Char('z'),
      toggle_repeat: ControlKey::Char('r'),
      volume_down: ControlKey::KeyCode(nc::KEY_LEFT),
      volume_up: ControlKey::KeyCode(nc::KEY_RIGHT),
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
      volume_change_step: 2,
    }
  }
}

impl Config {
  pub fn new() -> Config {
    // TODO: support MPD_SOCK
    // let addr = env::var("MPD_SOCK").unwrap_or("127.0.0.1:6600".to_owned());

    // Search for the MPD_PORT environment variable
    let mpd_ip = "127.0.0.1".parse().unwrap();
    let mpd_port = env::var("MPD_PORT")
      .unwrap_or("6600".to_owned())
      .parse::<u16>()
      .unwrap_or(6600);
    println!("MPD: {}:{}", mpd_ip, mpd_port);

    let keys = KeyConfig::new();
    let params = ParamConfig::new();

    Config {
      colors: ColorConfig::new(),
      addr: SocketAddr::new(mpd_ip, mpd_port),
      keys: keys,
      params: params,
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
    "volume_change_step" => config.params.volume_change_step = parse_int(val),
    "display_bitrate" => config.params.display_bitrate = parse_bool(val),
    "display_remaining_time" => config.params.display_remaining_time = parse_bool(val),
    "display_volume_level" => config.params.display_volume_level = parse_bool(val),
    "header_text_scrolling" => config.params.header_text_scrolling = parse_bool(val),
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
      None => {}
    }

    ConfigLoader { default_config_path: default_config_path }
  }

  pub fn load(&self, opt_path: Option<PathBuf>) -> Config {
    let path = match opt_path {
      Some(x) => x,
      None => self.default_config_path.clone(),
    };

    let mut config = Config::new();

    // Read ncmpcpp configuration
    let i = Ini::load_from_file(path.to_str().unwrap()).unwrap();
    for (_, prop) in i.iter() {
      for (k, v) in prop.iter() {
        // Remove quotes
        let fixed = v.trim_matches('\"');
        assign(&k, fixed, &mut config);
      }
    }

    return config;
  }
}
