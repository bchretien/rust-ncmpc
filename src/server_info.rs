extern crate mpd;

use chrono::{DateTime, Local, TimeZone};
use config::ParamConfig;
use constants::*;
use ncurses as nc;
use std::net::TcpStream;
use time::{Duration, Timespec};
use view::bold;

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
    if !s.is_empty() {
      s.push_str(", ");
    }
    s.push_str(&format!("{}h", hours));
  }

  if minutes > 0 {
    if !s.is_empty() {
      s.push_str(", ");
    }
    s.push_str(&format!("{}m", minutes));
  }

  if seconds > 0 {
    if !s.is_empty() {
      s.push_str(", ");
    }
    s.push_str(&format!("{}s", seconds));
  }

  return s;
}

/// Format duration for server info, e.g.:
/// 0:02
/// 5:57:53
fn format_duration_time(duration: &Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes() - hours * 60;
  let seconds = duration.num_seconds() - (hours * 60 + minutes) * 60;
  if hours == 0 {
    return format!("{}:{:02}", minutes, seconds);
  } else {
    return format!("{}:{}:{:02}", hours, minutes, seconds);
  }
}

/// Format date for server info, e.g.:
/// 11/26/2014 07:51:29 PM
fn format_date(ts: &Timespec) -> String {
  let date: DateTime<Local> = Local.timestamp(ts.sec, ts.nsec as u32);
  return date.format("%m/%d/%Y %I:%M:%S %p").to_string();
}

pub struct ServerInfo {
  border_pad: nc::WINDOW,
  pad: nc::WINDOW,
  max_x: i32,
  max_y: i32,
  current_row: i32,
  total_rows: i32,
  tab_size: i32,
  border_color: nc::attr_t,
}

impl ServerInfo {
  pub fn new(win: nc::WINDOW, _config: &ParamConfig) -> ServerInfo {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(win, &mut max_y, &mut max_x);
    max_x = (0.6 * (max_x as f32)) as i32;
    max_y = (0.8 * (max_y as f32)) as i32;
    ServerInfo {
      border_pad: nc::newpad(max_y + 2, max_x + 2),
      pad: nc::newpad(max_y, max_x),
      max_x: max_x,
      max_y: max_y,
      current_row: 0,
      total_rows: 0,
      tab_size: 0,
      border_color: nc::COLOR_PAIR(COLOR_PAIR_WINDOW_BORDER),
    }
  }

  fn newline(&mut self) {
    self.current_row += 1;
  }

  fn title(&mut self, name: &str) {
    nc::wattron(self.pad, self.border_color);
    nc::wattron(self.pad, bold());
    nc::mvwprintw(self.pad, self.current_row, self.tab_size, &name);
    nc::wattroff(self.pad, bold());
    nc::mvwhline(self.pad, 1, 0, nc::ACS_HLINE(), self.max_x);
    nc::wattroff(self.pad, self.border_color);
    self.current_row += 2;

    // Fix intersection with border pad
    nc::wattron(self.border_pad, self.border_color);
    nc::mvwprintw(self.border_pad, 2, 0, "├");
    nc::mvwprintw(self.border_pad, 2, self.max_x + 1, "┤");
    nc::wattroff(self.border_pad, self.border_color);
  }

  fn entry(&mut self, name: &str, value: &str) {
    let len = name.len() as i32;
    nc::wattron(self.pad, bold());
    nc::mvwprintw(self.pad, self.current_row, 2 * self.tab_size, &name);
    nc::mvwprintw(self.pad, self.current_row, 2 * self.tab_size + len, ": ");
    nc::wattroff(self.pad, bold());
    nc::mvwprintw(self.pad, self.current_row, 2 * self.tab_size + len + 2, &value);
    self.current_row += 1;
  }

  pub fn print(&mut self, client: &mut mpd::Client<TcpStream>) {
    nc::wclear(self.pad);
    self.current_row = 0;

    let stats: &mpd::Stats = &client.stats().unwrap_or_default();

    // Draw box around pad
    nc::wattron(self.border_pad, self.border_color);
    nc::box_(self.border_pad, 0, 0);
    nc::wattroff(self.border_pad, self.border_color);

    self.title("MPD server info");

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
    self.entry("Last DB update", format_date(&stats.db_update).as_str());
    self.newline();
    let url_handlers = client.urlhandlers();
    if url_handlers.is_ok() {
      self.entry("URL Handlers", url_handlers.unwrap().join(", ").as_str());
      self.newline();
    }

    self.newline();
    let tag_types = client.tagtypes();
    if tag_types.is_ok() {
      self.entry("Tag Types", tag_types.unwrap().join(", ").as_str());
    }

    if self.total_rows == 0 {
      self.total_rows = self.current_row;
    }

    let x_offset = (0.30 * (self.max_x as f32)) as i32;
    let y_offset = (0.20 * (self.max_y as f32)) as i32;
    nc::prefresh(
      self.border_pad,
      0,
      0,
      y_offset - 1,
      x_offset - 1,
      y_offset + 1 + self.max_y,
      x_offset + 1 + self.max_x,
    );
    nc::prefresh(self.pad, 0, 0, y_offset, x_offset, y_offset + self.max_y, x_offset + self.max_x);
  }
}
