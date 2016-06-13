extern crate ncurses;
extern crate ini;
extern crate xdg;

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use std::path::PathBuf;
use std::collections::HashMap;

use ini::Ini;
use constants::*;
use parser::parse_bindings_configuration;
use format::{Column, generate_columns};
use ncurses as nc;

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum ControlKey {
  KeyCode(i32),
  Char(char),
}

pub type ControlKeys = Vec<ControlKey>;
pub type CustomActions = HashMap<i32, Vec<String>>;

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
  pub custom: CustomActions,
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
  pub progressbar_look: String,
  pub song_columns_list_format: Vec<Column>,
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
      custom: CustomActions::default(),
    }
  }
}

fn to_keycode(key: &str) -> i32 {
  if key.chars().count() == 1 {
    return key.chars().next().unwrap() as i32;
  } else {
    // ctrl-?
    if key.len() == 6 && (key.starts_with("ctrl_") || key.starts_with("ctrl-")) {
      let next: char = key.chars().skip(5).next().unwrap();
      if next >= 'a' && next <= 'z' {
        return 1 + (next as i32 - 'a' as i32);
      } else if next == '[' {
        return KEY_CTRL_LEFTBRACKET;
      } else if next == '\\' {
        return KEY_CTRL_BACKSLASH;
      } else if next == ']' {
        return KEY_CTRL_RIGHTBRACKET;
      } else if next == '^' {
        return KEY_CTRL_CARET;
      } else if next == '_' {
        return KEY_CTRL_UNDERSCORE;
      }
      // Discard control qualifier
      return next as i32;
    }
    // shift-?
    else if key.starts_with("shift_") {
      return nc::KEY_UP;
    }
    // f?
    else if key.starts_with("f") {
      let other = key.chars().skip(1).next();
      return nc::KEY_UP;
    }
    // TODO: use a hashmap for the rest
    else if key == "escape" {
      return KEY_ESCAPE;
    } else if key == "up" {
      return nc::KEY_UP;
    } else if key == "down" {
      return nc::KEY_DOWN;
    } else if key == "left" {
      return nc::KEY_LEFT;
    } else if key == "right" {
      return nc::KEY_RIGHT;
    } else if key == "page_up" {
      return nc::KEY_PPAGE;
    } else if key == "page_down" {
      return nc::KEY_NPAGE;
    } else if key == "home" {
      return nc::KEY_HOME;
    } else if key == "end" {
      return nc::KEY_END;
    } else if key == "space" {
      return ' ' as i32;
    } else if key == "insert" {
      return nc::KEY_IC;
    } else if key == "delete" {
      return nc::KEY_DC;
    } else if key == "tab" {
      return KEY_TAB;
    } else if key == "backspace" {
      return KEY_BACKSPACE;
    }
    return -1;
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

fn default_song_columns_list_format() -> Vec<Column> {
  return generate_columns("(20)[]{a} (6f)[green]{NE} (50)[white]{t|f:Title} (20)[cyan]{b} \
                           (7f)[magenta]{l}")
    .unwrap();
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
      progressbar_look: String::from("=>"),
      song_columns_list_format: default_song_columns_list_format(),
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
    // TODO: add check (size 2 or 3)
    "progressbar_look" => config.params.progressbar_look = String::from(val),
    "volume_change_step" => config.params.volume_change_step = parse_int(val),
    // Formats
    "song_columns_list_format" => {
      config.params.song_columns_list_format = generate_columns(val)
        .unwrap_or(Vec::<Column>::default())
    }
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

  pub fn load(&self, user_config: Option<PathBuf>, bindings: Option<PathBuf>) -> Config {
    let opt_config =
      if user_config.is_some() { user_config.clone() } else { self.default_config_path.clone() };
    let opt_bindings =
      if bindings.is_some() { bindings.clone() } else { self.default_bindings_path.clone() };

    let mut config = Config::new();

    // Read ncmpcpp configuration (.ini file)
    if opt_config.is_some() {
      let path = opt_config.unwrap();
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

    // Read ncmpcpp bindings
    if opt_bindings.is_some() {
      let path = opt_bindings.unwrap();
      let res = parse_bindings_configuration(&PathBuf::from(path.to_str().unwrap()));
      if res.is_ok() {
        for val in res.unwrap() {
          let key = to_keycode(val.0.as_str());
          config.keys.custom.insert(key, val.1);
        }
      } else {
        stderr!("[Error] failed to parse {}", path.to_str().unwrap());
      }
    }

    return config;
  }
}

#[test]
fn test_keycode() {
  assert_eq!(to_keycode("a"), 'a' as i32);
  assert_eq!(to_keycode("z"), 'z' as i32);
  assert_eq!(to_keycode("escape"), KEY_ESCAPE);
  assert_eq!(to_keycode("left"), nc::KEY_LEFT);
  assert_eq!(to_keycode("right"), nc::KEY_RIGHT);
  assert_eq!(to_keycode("up"), nc::KEY_UP);
  assert_eq!(to_keycode("down"), nc::KEY_DOWN);
  assert_eq!(to_keycode("page_up"), nc::KEY_PPAGE);
  assert_eq!(to_keycode("page_down"), nc::KEY_NPAGE);
  assert_eq!(to_keycode("tab"), KEY_TAB);
  assert_eq!(to_keycode("ctrl_a"), 1);
  assert_eq!(to_keycode("ctrl_z"), 26);
  assert_eq!(to_keycode("ctrl_["), KEY_CTRL_LEFTBRACKET);
  assert_eq!(to_keycode("ctrl_\\"), KEY_CTRL_BACKSLASH);
  assert_eq!(to_keycode("ctrl_]"), KEY_CTRL_RIGHTBRACKET);
  assert_eq!(to_keycode("ctrl_^"), KEY_CTRL_CARET);
  assert_eq!(to_keycode("ctrl__"), KEY_CTRL_UNDERSCORE);
  assert_eq!(to_keycode("ctrl-a"), 1);
  assert_eq!(to_keycode("ctrl-z"), 26);
  assert_eq!(to_keycode("ctrl-["), KEY_CTRL_LEFTBRACKET);
  assert_eq!(to_keycode("ctrl-\\"), KEY_CTRL_BACKSLASH);
  assert_eq!(to_keycode("ctrl-]"), KEY_CTRL_RIGHTBRACKET);
  assert_eq!(to_keycode("ctrl-^"), KEY_CTRL_CARET);
  assert_eq!(to_keycode("ctrl-_"), KEY_CTRL_UNDERSCORE);
}
