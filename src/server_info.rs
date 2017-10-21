extern crate mpd;

use config::{ParamConfig, ControlKeys};
use ncurses as nc;
use std::net::TcpStream;
use time::Duration;

use view::{bold, get_color};

/// Format duration for server info, e.g.:
/// 34d, 5h, 57m, 53s
fn format_duration(duration: &Duration) -> String {
  let mut s = String::from("");
  let days = duration.num_days();
  let hours = duration.num_hours() - days * 24;
  let minutes = duration.num_minutes() - (days * 24 + hours) * 60;
  let seconds = duration.num_seconds() - ((days * 24 + hours) * 60 + minutes) * 60;

  if days > 0 {
    s.push_str(&format!("{}d", days));
  }

  if hours > 0 {
    if s.len() > 0 {
      s.push_str(", ");
    }
    s.push_str(&format!("{}h", hours));
  }

  if minutes > 0 {
    if s.len() > 0 {
      s.push_str(", ");
    }
    s.push_str(&format!("{}m", minutes));
  }

  if seconds > 0 {
    if s.len() > 0 {
      s.push_str(", ");
    }
    s.push_str(&format!("{}s", seconds));
  }

  return s;
}

/// Format duration for server info, e.g.:
/// 5:57:53
fn format_duration_time(duration: &Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes() - hours * 60;
  let seconds = duration.num_seconds() - (hours * 60 + minutes) * 60;
  return format!("{}:{}:{}", hours, minutes, seconds);
}

pub struct ServerInfo {
  win: nc::WINDOW,
  config: ParamConfig,
  current_row: i32,
  tab_size: i32,
}

impl ServerInfo {
  pub fn new(win: nc::WINDOW, config: &ParamConfig) -> ServerInfo {
    ServerInfo {
      win: win,
      config: config.clone(),
      current_row: 0,
      tab_size: 2,
    }
  }

  fn newline(&mut self) {
    self.current_row += 1;
  }

  fn title(&mut self, name: &str) {
    nc::wattron(self.win, bold());
    nc::mvwprintw(self.win, self.current_row, self.tab_size, &name);
    nc::wattroff(self.win, bold());
    self.current_row += 1;
  }

  fn entry(&mut self, name: &str, value: &str) {
    let len = name.len() as i32;
    nc::wattron(self.win, bold());
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size, &name);
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size + len, ": ");
    nc::wattroff(self.win, bold());
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size + len + 2, &value);
    self.current_row += 1;
  }

  pub fn print(&mut self, client: &mut mpd::Client<TcpStream>) {
    nc::wclear(self.win);
    self.current_row = 0;

    let stats: &mpd::Stats = &client.stats().unwrap_or(mpd::Stats::default());

    self.newline();
    self.title("MPD server info");
    self.newline();
    self.entry(
      "Version",
      &format!("{}.{}.{}", client.version.0, client.version.1, client.version.2),
    );
    self.entry("Uptime", format_duration(&stats.uptime).as_str());
    self.entry("Time playing", format_duration_time(&stats.playtime).as_str());
    self.newline();
    self.entry("Total playtime", format_duration(&stats.db_playtime).as_str());
    self.entry("Artist names", &format!("{}", stats.artists));
    self.entry("Album names", &format!("{}", stats.albums));
    self.entry("Songs in database", &format!("{}", stats.songs));
    self.newline();
    self.entry("Last DB update", &format!("{:?}", stats.db_update));
    self.newline();
    let url_handlers = client.urlhandlers();
    if url_handlers.is_ok() {
      self.entry("URL Handlers", url_handlers.unwrap().join(", ").as_str());
    }
    self.newline();
    let tag_types = client.tagtypes();
    if tag_types.is_ok() {
      self.entry("Tag Types", tag_types.unwrap().join(", ").as_str());
    }

    nc::wrefresh(self.win);
  }
}
