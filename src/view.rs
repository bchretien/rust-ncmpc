extern crate ncurses;

use ncurses as nc;

use std::{cmp, iter, mem};
use std::fmt::{self, Display, Formatter};
use time::Duration;

use constants::*;
use config::ColorConfig;
use util::Scroller;

pub struct PlaylistData {
  pub size: u32,
  pub duration: Duration,
}

impl PlaylistData {
  pub fn new() -> PlaylistData {
    PlaylistData {
      size: 0,
      duration: Duration::seconds(0),
    }
  }
}

impl Display for PlaylistData {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.size == 0 {
      write!(f, "0 item")
    } else {
      let n_h = self.duration.num_hours();
      let n_min = self.duration.num_minutes() % 60;
      let n_sec = self.duration.num_seconds() % 60;
      let s_h = if n_h > 1 { "s" } else { "" };
      let s_min = if n_min > 1 { "s" } else { "" };
      let s_sec = if n_sec > 1 { "s" } else { "" };
      write!(f,
             "{} items, length: {} hour{}, {} minute{}, {} second{}",
             self.size,
             n_h,
             s_h,
             n_min,
             s_min,
             n_sec,
             s_sec)
    }
  }
}

pub enum MouseEvent {
  // Set the song progress (percentage)
  SetProgress(f32),
  // Do nothing
  Nothing,
}

pub struct View {
  header: nc::WINDOW,
  header_scroller: Scroller,
  state: nc::WINDOW,
  main_win: nc::WINDOW,
  progressbar: nc::WINDOW,
  statusbar: nc::WINDOW,
  status_scroller: Scroller,
  debug_row: nc::WINDOW,
  static_rows: i32,
}

fn init_colors(colors: &ColorConfig) {
  nc::start_color();

  // Background transparency
  let mut color_bg = nc::COLOR_BLACK;
  let mut color_fg = nc::COLOR_WHITE;
  if nc::use_default_colors() == nc::OK {
    color_fg = -1;
    color_bg = -1;
  }

  nc::init_pair(COLOR_PAIR_DEFAULT, color_fg, color_bg);
  nc::init_pair(COLOR_PAIR_ARTIST, nc::COLOR_YELLOW, color_bg);
  nc::init_pair(COLOR_PAIR_HEADER, colors.header_window, color_bg);
  nc::init_pair(COLOR_PAIR_PROGRESSBAR, colors.progressbar, color_bg);
  nc::init_pair(COLOR_PAIR_PROGRESSBAR_ELAPSED, colors.progressbar_elapsed, color_bg);
  nc::init_pair(COLOR_PAIR_STATUSBAR, colors.statusbar, color_bg);
  nc::init_pair(COLOR_PAIR_VOLUME, colors.volume, color_bg);
  nc::init_pair(COLOR_PAIR_DEBUG, nc::COLOR_GREEN, color_bg);
  nc::init_pair(COLOR_PAIR_STATE_LINE, colors.state_line, color_bg);
  nc::init_pair(COLOR_PAIR_STATE_FLAGS, colors.state_flags, color_bg);
  nc::init_pair(COLOR_PAIR_TRACK, nc::COLOR_BLACK, color_bg);
}

fn init_ncurses(colors: &ColorConfig) {
  // Set locale for unicode support.
  let locale_conf = nc::LcCategory::all;
  nc::setlocale(locale_conf, "en_US.UTF-8");

  // Start ncurses.
  nc::initscr();

  // Initialize colors.
  init_colors(colors);

  // Make cursor invisible.
  nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
  nc::cbreak();

  // Allow for extended keyboard (like F1).
  nc::keypad(nc::stdscr, true);
  nc::noecho();

  // Set timeout.
  nc::timeout(0);

  // Make getch non-blocking.
  nc::nodelay(nc::stdscr, true);

  // Enable mouse events.
  nc::mousemask(nc::BUTTON1_PRESSED as u64, None);

  nc::clear();
}

// TODO: check 32/64 bit attr_t
fn get_color(c: i16) -> i32 {
  return nc::COLOR_PAIR(c) as i32;
}

fn bold() -> i32 {
  return nc::A_BOLD() as i32;
}

fn deinit_ncurses() {
  // Terminate ncurses.
  nc::endwin();
}

fn destroy_win(win: nc::WINDOW) {
  let ch = ' ' as nc::chtype;
  nc::wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
  nc::wrefresh(win);
  nc::delwin(win);
}

impl View {
  pub fn new(colors: &ColorConfig) -> View {
    init_ncurses(colors);

    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);
    let static_rows = 5;

    let view = View {
      header: nc::newwin(1, max_x, 0, 0),
      header_scroller: Scroller::new(max_x as usize),
      state: nc::newwin(1, max_x, 1, 0),
      main_win: nc::newwin(max_y - static_rows, max_x, 2, 0),
      progressbar: nc::newwin(1, max_x, max_y - 3, 0),
      statusbar: nc::newwin(1, max_x, max_y - 2, 0),
      status_scroller: Scroller::new(max_x as usize),
      debug_row: nc::newwin(1, max_x, max_y - 1, 0),
      static_rows: static_rows,
    };
    nc::wrefresh(view.header);
    nc::wrefresh(view.state);
    nc::wrefresh(view.main_win);
    nc::wrefresh(view.progressbar);
    nc::wrefresh(view.statusbar);
    nc::wrefresh(view.debug_row);
    nc::keypad(view.main_win, true);

    // Set colors
    nc::wbkgd(view.header, nc::COLOR_PAIR(COLOR_PAIR_ARTIST) as nc::chtype);
    nc::wbkgd(view.state, nc::COLOR_PAIR(COLOR_PAIR_DEFAULT) as nc::chtype);
    nc::wbkgd(view.debug_row, nc::COLOR_PAIR(COLOR_PAIR_DEBUG) as nc::chtype);

    return view;
  }

  pub fn display_header(&mut self, pl_data: &PlaylistData, volume: Option<i8>) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.header, &mut max_y, &mut max_x);
    let mut free_size = max_x;

    // Clear
    nc::wmove(self.header, 0, 0);
    nc::wclrtoeol(self.header);

    // Start of the header
    let title = "Playlist";
    let pl_color = get_color(COLOR_PAIR_HEADER);
    nc::wattron(self.header, pl_color);
    nc::wattron(self.header, bold());
    nc::mvwprintw(self.header, 0, 0, &title);
    nc::wattroff(self.header, bold());
    nc::wattroff(self.header, pl_color);
    free_size -= title.len() as i32;

    // Volume
    if volume.is_some() {
      let vol_color = get_color(COLOR_PAIR_VOLUME);
      nc::wattron(self.header, vol_color);
      let s = format!(" Volume: {}%%", volume.unwrap());
      nc::mvwprintw(self.header, 0, 1 + max_x - s.len() as i32, s.as_str());
      nc::wattroff(self.header, vol_color);
      free_size -= s.len() as i32;
    }

    // Playlist details
    let s = format!("({})", pl_data);
    // TODO: only change text on playlist change
    self.header_scroller.set_text(&s);
    self.header_scroller.resize(free_size);
    nc::wattron(self.header, pl_color);
    nc::wattron(self.header, bold());
    nc::mvwprintw(self.header, 0, 1 + title.len() as i32, self.header_scroller.display());
    nc::wattroff(self.header, bold());
    nc::wattroff(self.header, pl_color);

    nc::wrefresh(self.header);
  }

  // TODO: data should not be mutable
  pub fn display_main_playlist(&mut self, desc: &[(String, u32)], data: &mut [&mut [String]], current_song: Option<u32>) {
    // Get the screen bounds.
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.main_win, &mut max_y, &mut max_x);

    let mut color = get_color(COLOR_PAIR_DEFAULT);

    nc::wattron(self.main_win, bold());
    nc::wattron(self.main_win, color);

    // Header
    let mut x = 0;
    for col in desc {
      nc::wmove(self.main_win, 0, x - 1);
      nc::wclrtoeol(self.main_win);
      nc::mvwprintw(self.main_win, 0, x, &format!("{}", col.0));
      x += 1 + col.1 as i32;
    }

    // Separator
    nc::wmove(self.main_win, 1, 0);
    nc::wclrtoeol(self.main_win);
    let sep = iter::repeat('─').take(max_x as usize).collect::<String>();
    nc::mvwprintw(self.main_win, 1, 0, &sep);

    nc::wattroff(self.main_win, color);
    nc::wattroff(self.main_win, bold());

    // Playlist data
    let pl_start_row = 2;
    let pl_max_row = max_y;

    color = get_color(COLOR_PAIR_ARTIST);
    nc::wattron(self.main_win, color);

    let height = cmp::min(pl_max_row - pl_start_row, data.len() as i32);
    // For each song
    for y in 0..height {
      // For each column
      x = 0;
      for i in 0..desc.len() {
        nc::wmove(self.main_win, pl_start_row + y, cmp::max(x - 1, 0));
        nc::wclrtoeol(self.main_win);

        // Highlight current song
        let is_current = current_song.is_some() && current_song.unwrap() == y as u32;
        if is_current {
          nc::wattron(self.main_win, bold());
        }
        nc::mvwprintw(self.main_win, pl_start_row + y, x, &format!("{}", data[y as usize][i as usize]));

        // Stop highlighting
        if is_current {
          nc::wattroff(self.main_win, bold());
        }

        x += 1 + desc[i].1 as i32;
      }
    }
    // Clear the rest of the lines
    for y in height..max_y - 3 {
      nc::wmove(self.main_win, pl_start_row + y, 0);
      nc::wclrtoeol(self.main_win);
    }
    nc::wattroff(self.main_win, color);

    nc::wrefresh(self.main_win);
  }

  pub fn display_progressbar(&mut self, pct: f32) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.progressbar, &mut max_y, &mut max_x);

    let tip_x: i32 = (pct / 100. * (max_x as f32)) as i32;

    // Start of the bar
    let len_start: usize = tip_x as usize;
    let sep = iter::repeat('─').take(len_start).collect::<String>();
    let mut color = get_color(COLOR_PAIR_PROGRESSBAR_ELAPSED);
    nc::wattron(self.progressbar, color);
    nc::mvwprintw(self.progressbar, 0, 0, &sep);

    if pct > 0. {
      // Tip of the bar
      let tip = "╼";
      nc::mvwprintw(self.progressbar, 0, tip_x, &tip);
      nc::wattroff(self.progressbar, color);
    }

    // End of the bar
    let len_end: usize = (max_x - tip_x) as usize;
    let sep = iter::repeat('─').take(len_end).collect::<String>();
    color = get_color(COLOR_PAIR_PROGRESSBAR);
    nc::wattron(self.progressbar, color);
    nc::mvwprintw(self.progressbar, 0, if tip_x > 0 { tip_x + 1 } else { 0 }, &sep);
    nc::wattroff(self.progressbar, color);

    nc::wrefresh(self.progressbar);
  }

  pub fn display_stateline(&mut self, flags: &Vec<char>) {
    // Clear line.
    nc::wmove(self.state, 0, 0);
    nc::wclrtoeol(self.state);

    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.state, &mut max_y, &mut max_x);

    // Print the bar
    let sep = iter::repeat('─').take(max_x as usize).collect::<String>();
    let mut color = get_color(COLOR_PAIR_STATE_LINE);
    nc::wattron(self.state, color);
    nc::mvwprintw(self.state, 0, 0, &sep);
    nc::wattroff(self.state, color);

    if !flags.is_empty() {
      let s: String = flags.iter().fold("".to_string(), |mut vec, val| {
        vec.push(val.clone());
        vec
      });

      // Print the brackets
      nc::wattron(self.state, color);
      nc::mvwprintw(self.state, 0, max_x - 3 - s.len() as i32, "[");
      nc::mvwprintw(self.state, 0, max_x - 2, "]");
      nc::wattroff(self.state, color);

      // Print the flags
      color = get_color(COLOR_PAIR_STATE_FLAGS);
      nc::wattron(self.state, color);
      nc::wattron(self.state, bold());
      nc::mvwprintw(self.state, 0, max_x - 2 - s.len() as i32, &s);
      nc::wattroff(self.state, bold());
      nc::wattroff(self.state, color);
    }

    nc::wrefresh(self.state);
  }

  pub fn display_statusbar(&mut self, mode: &str, msg: &str, track: &str) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.statusbar, &mut max_y, &mut max_x);

    let mut free_size = max_x;

    // Clear line.
    nc::wmove(self.statusbar, 0, 0);
    nc::wclrtoeol(self.statusbar);

    // Print mode.
    if !mode.is_empty() {
      let color = get_color(COLOR_PAIR_STATUSBAR);
      nc::wattron(self.statusbar, color);
      nc::wattron(self.statusbar, bold());
      nc::mvwprintw(self.statusbar, 0, 0, &format!("{}:", mode));
      nc::wattroff(self.statusbar, bold());
      nc::wattroff(self.statusbar, color);

      free_size -= mode.len() as i32 + 2;
    }

    // Print track (time, bitrate, etc.)
    if !track.is_empty() {
      let color = get_color(COLOR_PAIR_TRACK);
      nc::wattron(self.statusbar, color);
      nc::wattron(self.statusbar, bold());
      let offset = max_x - track.len() as i32;
      nc::mvwprintw(self.statusbar, 0, offset - 1, " ");
      nc::mvwprintw(self.statusbar, 0, offset, track);
      nc::wattroff(self.statusbar, bold());
      nc::wattroff(self.statusbar, color);

      free_size -= track.len() as i32 + 1;
    }

    // Print message.
    // TODO: only change text on song change
    self.status_scroller.set_text(msg);
    self.status_scroller.resize(free_size);
    let color = get_color(COLOR_PAIR_DEFAULT);
    nc::wattron(self.statusbar, color);
    let offset = mode.len() + 2;
    nc::mvwprintw(self.statusbar, 0, offset as i32, self.status_scroller.display());
    nc::wattroff(self.statusbar, color);

    nc::wrefresh(self.statusbar);
  }

  pub fn display_debug_prompt(&mut self, msg: &str) {
    // Clear line.
    nc::wmove(self.debug_row, 0, 0);
    nc::wclrtoeol(self.debug_row);
    // Print message.
    nc::mvwprintw(self.debug_row, 0, 0, &format!("[Debug] {}", msg));

    nc::wrefresh(self.debug_row);
  }

  pub fn process_mouse(&mut self) -> MouseEvent {
    let mut event: nc::MEVENT = unsafe { mem::uninitialized() };
    if nc::getmouse(&mut event) == nc::OK {
      let mut max_x = 0;
      let mut max_y = 0;
      let mut win_x = 0;
      let mut win_y = 0;

      // Check progressbar event
      nc::getbegyx(self.progressbar, &mut win_y, &mut win_x);
      if event.y == win_y {
        nc::getmaxyx(self.progressbar, &mut max_y, &mut max_x);
        return MouseEvent::SetProgress(event.x as f32 / max_x as f32);
      }
    }
    return MouseEvent::Nothing;
  }

  pub fn resize_windows(&mut self) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

    let mut row = 0;
    nc::wresize(self.header, 1, max_x);
    nc::mvwin(self.header, row, 0);
    row += 1;

    nc::wresize(self.state, 1, max_x);
    nc::mvwin(self.state, row, 0);
    row += 1;

    nc::wresize(self.main_win, max_y - self.static_rows, max_x);
    nc::mvwin(self.main_win, row, 0);
    row += max_y - self.static_rows;

    nc::wresize(self.progressbar, 1, max_x);
    nc::mvwin(self.progressbar, row, 0);
    row += 1;

    nc::wresize(self.statusbar, 1, max_x);
    nc::mvwin(self.statusbar, row, 0);
    row += 1;

    nc::wresize(self.debug_row, 1, max_x);
    nc::mvwin(self.debug_row, row, 0);

    // TODO: resize scrollers?
    // self.header_scroller.resize();

    nc::refresh();
  }
}

impl Drop for View {
  fn drop(&mut self) {
    destroy_win(self.header);
    destroy_win(self.state);
    destroy_win(self.main_win);
    destroy_win(self.progressbar);
    destroy_win(self.statusbar);
    destroy_win(self.debug_row);
    deinit_ncurses();
  }
}
