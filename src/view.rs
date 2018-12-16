extern crate mpd;
extern crate ncurses;

use crate::config::{ColorConfig, Config, ParamConfig};

use crate::constants::*;
use crate::format::*;
use crate::help::*;
use crate::server_info::*;
use ncurses as nc;

use crate::util::{Scroller, TimedValue};
use std::fmt::{self, Display, Formatter};
use std::net::TcpStream;
use std::{char, cmp, mem};
use time::{get_time, Duration, Timespec};

pub struct PlaylistInfo {
  pub size: u32,
  pub duration: Duration,
}

impl PlaylistInfo {
  pub fn new() -> PlaylistInfo {
    PlaylistInfo {
      size: 0,
      duration: Duration::seconds(0),
    }
  }
}

impl Default for PlaylistInfo {
  fn default() -> Self {
    Self::new()
  }
}

impl Display for PlaylistInfo {
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
      write!(
        f,
        "{} items, length: {} hour{}, {} minute{}, {} second{}",
        self.size, n_h, s_h, n_min, s_min, n_sec, s_sec
      )
    }
  }
}

pub enum MouseEvent {
  /// Set the song progress (percentage).
  SetProgress(f32),
  /// Set the selected song (TUI).
  SetSelectedSong(u32),
  /// Scroll down.
  ScrollDown,
  /// Scroll up.
  ScrollUp,
  /// Wake up click (re-highlight selected song).
  WakeUp,
  /// Do nothing.
  Nothing,
}

#[derive(Debug, PartialEq)]
pub enum ActiveWindow {
  /// Displaying help window.
  Help,
  /// Displaying current playlist.
  Playlist,
  /// Displaying MPD server info.
  ServerInfo,
}

pub struct View {
  header: nc::WINDOW,
  header_scroller: Scroller,
  state: nc::WINDOW,
  main_win: nc::WINDOW,
  progressbar: nc::WINDOW,
  progressbar_look: Vec<String>,
  statusbar: nc::WINDOW,
  statusbar_input: Vec<String>,
  pub help: Help,
  server_info: ServerInfo,
  status_scroller: Scroller,
  static_rows: i32,
}

fn init_colors(colors: &ColorConfig, params: &ParamConfig) {
  nc::start_color();

  // Background transparency
  let mut color_bg = nc::COLOR_BLACK;
  let mut color_fg = nc::COLOR_WHITE;
  if nc::use_default_colors() == nc::OK {
    color_fg = -1;
    color_bg = -1;
  }

  nc::init_pair(COLOR_PAIR_DEFAULT, color_fg, color_bg);
  nc::init_pair(COLOR_PAIR_BLACK, nc::COLOR_BLACK, color_bg);
  nc::init_pair(COLOR_PAIR_RED, nc::COLOR_RED, color_bg);
  nc::init_pair(COLOR_PAIR_GREEN, nc::COLOR_GREEN, color_bg);
  nc::init_pair(COLOR_PAIR_YELLOW, nc::COLOR_YELLOW, color_bg);
  nc::init_pair(COLOR_PAIR_BLUE, nc::COLOR_BLUE, color_bg);
  nc::init_pair(COLOR_PAIR_MAGENTA, nc::COLOR_MAGENTA, color_bg);
  nc::init_pair(COLOR_PAIR_CYAN, nc::COLOR_CYAN, color_bg);
  nc::init_pair(COLOR_PAIR_WHITE, nc::COLOR_WHITE, color_bg);

  nc::init_pair(COLOR_PAIR_HEADER, colors.header_window, color_bg);
  nc::init_pair(COLOR_PAIR_PROGRESSBAR, colors.progressbar, color_bg);
  nc::init_pair(COLOR_PAIR_PROGRESSBAR_ELAPSED, colors.progressbar_elapsed, color_bg);
  nc::init_pair(COLOR_PAIR_STATUSBAR, colors.statusbar, color_bg);
  nc::init_pair(COLOR_PAIR_VOLUME, colors.volume, color_bg);
  nc::init_pair(COLOR_PAIR_STATE_LINE, colors.state_line, color_bg);
  nc::init_pair(COLOR_PAIR_STATE_FLAGS, colors.state_flags, color_bg);
  nc::init_pair(COLOR_PAIR_TRACK, nc::COLOR_BLACK, color_bg);
  nc::init_pair(COLOR_PAIR_WINDOW_BORDER, colors.window_border, color_bg);

  let columns_fmt = &params.song_columns_list_format;
  assert!(columns_fmt.len() <= MAX_NUM_COLUMNS);
  for (i, col) in columns_fmt.iter().enumerate() {
    nc::init_pair(COLOR_PAIR_COLUMNS[i as usize], col.color, color_bg);
  }
}

fn init_ncurses(config: &Config) {
  // Set locale for unicode support.
  let locale_conf = nc::LcCategory::all;
  nc::setlocale(locale_conf, "en_US.UTF-8");

  // Start ncurses.
  nc::initscr();

  // Initialize colors.
  init_colors(&config.colors, &config.params);

  // Make cursor invisible.
  nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
  nc::cbreak();

  // Allow for extended keyboard (like F1).
  nc::keypad(nc::stdscr(), true);
  nc::noecho();

  // Set timeout.
  nc::timeout(0);

  // Make getch non-blocking.
  nc::nodelay(nc::stdscr(), true);

  // Enable mouse events.
  nc::mouseinterval(0);
  nc::mousemask(
    (nc::BUTTON1_CLICKED | nc::BUTTON4_PRESSED | nc::BUTTON5_PRESSED) as nc::mmask_t,
    None,
  );

  nc::clear();
}

pub fn get_color(c: Color) -> nc::attr_t {
  return nc::COLOR_PAIR(c) as nc::attr_t;
}

pub fn bold() -> nc::attr_t {
  return nc::A_BOLD() as nc::attr_t;
}

fn reverse() -> nc::attr_t {
  return nc::A_REVERSE() as nc::attr_t;
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
  pub fn new(config: &Config) -> View {
    init_ncurses(config);

    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(nc::stdscr(), &mut max_y, &mut max_x);
    let static_rows = 4;

    let main_win = nc::newwin(max_y - static_rows, max_x, 2, 0);
    let view = View {
      header: nc::newwin(1, max_x, 0, 0),
      header_scroller: Scroller::new(max_x as usize),
      state: nc::newwin(1, max_x, 1, 0),
      main_win: main_win,
      progressbar: nc::newwin(1, max_x, max_y - 2, 0),
      progressbar_look: {
        let mut iter = config.params.progressbar_look.chars();
        let mut ar = vec![String::default(); 3];
        for elt in ar.iter_mut().take(3) {
          *elt = match iter.next() {
            Some(c) => c.to_string(),
            None => String::default(),
          }
        }
        ar
      },
      statusbar: nc::newwin(1, max_x, max_y - 1, 0),
      statusbar_input: vec![],
      help: Help::new(main_win, config),
      server_info: ServerInfo::new(main_win, &config.params),
      status_scroller: Scroller::new(max_x as usize),
      static_rows: static_rows,
    };
    nc::wrefresh(view.header);
    nc::wrefresh(view.state);
    nc::wrefresh(view.main_win);
    nc::wrefresh(view.progressbar);
    nc::wrefresh(view.statusbar);
    nc::keypad(view.main_win, true);

    // Set colors
    nc::wbkgd(view.header, nc::COLOR_PAIR(COLOR_PAIR_DEFAULT) as nc::chtype);
    nc::wbkgd(view.state, nc::COLOR_PAIR(COLOR_PAIR_DEFAULT) as nc::chtype);

    return view;
  }

  pub fn display_header(&mut self, active_window: &ActiveWindow, pl_info: &PlaylistInfo, volume: Option<i8>) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.header, &mut max_y, &mut max_x);
    let mut free_size = max_x;

    // Clear
    nc::wmove(self.header, 0, 0);
    nc::wclrtoeol(self.header);

    // Start of the header
    let title = format!("{:?}", active_window);
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
      let vol_value = volume.unwrap();
      let s: String = if vol_value >= 0 {
        format!(" Volume: {}%%", volume.unwrap())
      } else {
        String::from(" Volume: n/a")
      };

      nc::mvwprintw(self.header, 0, 1 + max_x - 1 - s.len() as i32, s.as_str());
      nc::wattroff(self.header, vol_color);
      free_size -= s.len() as i32;
    }

    if active_window == &ActiveWindow::Playlist {
      // Playlist details
      let s = format!("({})", pl_info);
      // TODO: only change text on playlist change
      self.header_scroller.set_text(&s);
      self.header_scroller.resize(free_size);
      nc::wattron(self.header, pl_color);
      nc::wattron(self.header, bold());
      nc::mvwprintw(self.header, 0, 1 + title.len() as i32, self.header_scroller.display());
      nc::wattroff(self.header, bold());
      nc::wattroff(self.header, pl_color);
    }

    nc::wrefresh(self.header);
  }

  pub fn display_help(&mut self) {
    self.help.print();
  }

  pub fn display_server_info(&mut self, client: &mut mpd::Client<TcpStream>) {
    self.server_info.print(client);
  }

  pub fn display_main_playlist(
    &self,
    desc: &[Column],
    data: &[&[String]],
    current_song: Option<u32>,
    selected_song: &Option<TimedValue<u32>>,
  ) {
    // Get the screen bounds.
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.main_win, &mut max_y, &mut max_x);

    // Evaluate absolute width of each column
    // First pass: look for fixed-width columns
    let mut widths = vec![0; desc.len()];
    let mut free_space = max_x;
    let mut relative_width = 0;
    for (i, c) in desc.iter().enumerate() {
      if c.is_fixed {
        widths[i] = c.width;
        free_space -= c.width;
      } else {
        relative_width += c.width;
      }
    }
    // Second pass: use relative width for remaining space
    for (i, c) in desc.iter().enumerate() {
      if !c.is_fixed {
        widths[i] = c.width * free_space / relative_width;
      }
    }

    let mut color = get_color(COLOR_PAIR_DEFAULT);

    nc::wattron(self.main_win, bold());
    nc::wattron(self.main_win, color);

    // Header
    let mut x = 0;
    for (i, col) in desc.iter().enumerate() {
      nc::wmove(self.main_win, 0, cmp::max(0, x - 1));
      nc::wclrtoeol(self.main_win);
      nc::mvwprintw(self.main_win, 0, x, &format!("{}", col.column_type));
      x += widths[i] as i32;
    }

    // Separator
    nc::wmove(self.main_win, 1, 0);
    nc::whline(self.main_win, nc::ACS_HLINE(), max_x);

    nc::wattroff(self.main_win, color);
    nc::wattroff(self.main_win, bold());

    // Playlist data
    let pl_start_row = 2;
    let pl_max_row = max_y;

    let highlight_ts: Timespec = match *selected_song {
      Some(s) => s.timestamp + Duration::seconds(5),
      None => Timespec::new(0, 0),
    };
    let highlighting: bool = get_time() < highlight_ts;

    // Total number of songs
    let n = data.len() as i32;
    // Index of the selected song
    let selected_idx = if selected_song.is_some() {
      selected_song.unwrap().value as i32
    } else {
      -1 as i32
    };
    // Maximum number of displayed song rows
    let max_height = pl_max_row - pl_start_row;
    // Number of displayed song rows
    let height = cmp::min(max_height, n);
    // Index of the song serving as the first displayed row
    let start_idx: i32 = if selected_idx < max_height / 2 {
      0
    } else if selected_idx < (n - max_height / 2) {
      selected_idx - max_height / 2
    } else {
      n - max_height
    };

    // For each song
    for (urow, ref item) in data.iter().skip(start_idx as usize).take(height as usize).enumerate() {
      let row: i32 = urow as i32;
      let idx: i32 = start_idx + row;
      // For each column
      x = 0;
      for i in 0..desc.len() as usize {
        nc::wmove(self.main_win, pl_start_row + row, x);
        nc::wclrtoeol(self.main_win);

        // Set column color
        color = get_color(COLOR_PAIR_COLUMNS[i]);
        nc::wattron(self.main_win, color);

        // Highlight current song
        let is_current = current_song.is_some() && current_song.unwrap() == idx as u32;
        if is_current {
          nc::wattron(self.main_win, bold());
        }

        // Highlight selected song
        let is_selected = highlighting && selected_idx == idx;
        if is_selected {
          nc::wattron(self.main_win, reverse());
        }

        // Print song
        nc::mvwprintw(self.main_win, pl_start_row + row, x, &item[i as usize]);

        // If it's not the last column
        if i < desc.len() - 1 {
          // Add whitespace before the next column
          nc::mvwaddch(self.main_win, pl_start_row + row, x + widths[i] - 1, ' ' as nc::chtype);
        }

        if is_selected {
          // Fill with whitespace for ncmpcpp-style highlighting
          let len = item[i as usize].chars().count() as i32;
          nc::mvwhline(self.main_win, pl_start_row + row, x + len, ' ' as nc::chtype, widths[i] - len);

          // Stop highlighting
          nc::wattroff(self.main_win, reverse());
        }

        // Stop highlighting current song
        if is_current {
          nc::wattroff(self.main_win, bold());
        }

        // Disable column color
        nc::wattroff(self.main_win, color);

        // TODO: handle variable width
        x += widths[i] as i32;
      }
    }

    // Clear the rest of the lines
    for y in height..max_height {
      nc::wmove(self.main_win, pl_start_row + y, 0);
      nc::wclrtoeol(self.main_win);
    }

    nc::wrefresh(self.main_win);
  }

  pub fn display_progressbar(&mut self, pct: f32) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.progressbar, &mut max_y, &mut max_x);

    let tip_x: i32 = (pct / 100. * (max_x as f32)) as i32;

    // Start of the bar
    let len_start = tip_x;
    let mut color = get_color(COLOR_PAIR_PROGRESSBAR_ELAPSED);
    nc::wattron(self.progressbar, color);
    // TODO: find why using mvwhline fails with ─
    if self.progressbar_look[0] == "─" {
      nc::mvwhline(self.progressbar, 0, 0, nc::ACS_HLINE(), len_start);
    } else {
      nc::wmove(self.progressbar, 0, 0);
      for _i in 0..len_start {
        nc::waddstr(self.progressbar, &self.progressbar_look[0]);
      }
    }

    if pct > 0. {
      // Tip of the bar
      nc::mvwprintw(self.progressbar, 0, tip_x, &self.progressbar_look[1]);
      nc::wattroff(self.progressbar, color);
    }

    // End of the bar
    let len_end = max_x - tip_x;
    color = get_color(COLOR_PAIR_PROGRESSBAR);
    nc::wattron(self.progressbar, color);
    if self.progressbar_look[2] == "─" {
      nc::mvwhline(self.progressbar, 0, if tip_x > 0 { tip_x + 1 } else { 0 }, nc::ACS_HLINE(), len_end);
    } else if self.progressbar_look[2] != "" {
      nc::wmove(self.progressbar, 0, if tip_x > 0 { tip_x + 1 } else { 0 });
      for _i in 0..len_end {
        nc::waddstr(self.progressbar, &self.progressbar_look[2]);
      }
    }
    nc::wattroff(self.progressbar, color);

    nc::wrefresh(self.progressbar);
  }

  pub fn display_stateline(&mut self, flags: &[char]) {
    // Clear line.
    nc::wmove(self.state, 0, 0);
    nc::wclrtoeol(self.state);

    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(self.state, &mut max_y, &mut max_x);

    // Print the bar
    let mut color = get_color(COLOR_PAIR_STATE_LINE);
    nc::wattron(self.state, color);
    nc::whline(self.state, nc::ACS_HLINE(), max_x);
    nc::wattroff(self.state, color);

    if !flags.is_empty() {
      let s: String = flags.iter().fold("".to_string(), |mut vec, val| {
        vec.push(*val);
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

    if !self.statusbar_input.is_empty() && !self.statusbar_input.last().unwrap().is_empty() {
      let color = get_color(COLOR_PAIR_DEFAULT);
      nc::wattron(self.statusbar, color);
      nc::mvwprintw(
        self.statusbar,
        0,
        0,
        format!(":{}", self.statusbar_input.last().unwrap().as_str()).as_str(),
      );
      nc::wattroff(self.statusbar, color);
    } else {
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
    }

    nc::wrefresh(self.statusbar);
  }

  pub fn display_statusbar_msg(&mut self, msg: &str) {
    nc::wmove(self.statusbar, 0, 0);
    nc::wclrtoeol(self.statusbar);
    nc::mvwprintw(self.statusbar, 0, 0, msg);
    nc::wrefresh(self.statusbar);
  }

  pub fn process_mouse(&mut self) -> MouseEvent {
    let mut event: nc::MEVENT = unsafe { mem::uninitialized() };
    if nc::getmouse(&mut event) == nc::OK {
      let mut max_x = 0;
      let mut max_y = 0;
      let mut win_x = 0;
      let mut win_y = 0;

      // Check playlist event
      nc::getbegyx(self.main_win, &mut win_y, &mut win_x);
      nc::getmaxyx(self.main_win, &mut max_y, &mut max_x);
      if event.y >= win_y + 2 && event.y < win_y + max_y {
        // Click
        if (event.bstate & (nc::BUTTON1_PRESSED as nc::mmask_t)) != 0 {
          return MouseEvent::SetSelectedSong((event.y - win_y) as u32 - 2);
        }
        // Mouse wheel up
        else if (event.bstate & (nc::BUTTON4_PRESSED as nc::mmask_t)) != 0 {
          return MouseEvent::ScrollUp;
        }
        // Mouse wheel down
        else if (event.bstate & (nc::BUTTON5_PRESSED as nc::mmask_t)) != 0 {
          return MouseEvent::ScrollDown;
        }
      }

      // Check progressbar event
      nc::getbegyx(self.progressbar, &mut win_y, &mut win_x);
      if event.y == win_y {
        nc::getmaxyx(self.progressbar, &mut max_y, &mut max_x);
        return MouseEvent::SetProgress(event.x as f32 / max_x as f32);
      }

      return MouseEvent::WakeUp;
    }
    return MouseEvent::Nothing;
  }

  pub fn resize_windows(&mut self) {
    let mut max_x = 0;
    let mut max_y = 0;
    nc::getmaxyx(nc::stdscr(), &mut max_y, &mut max_x);

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

    // TODO: resize scrollers?
    // self.header_scroller.resize();

    nc::refresh();
  }

  pub fn read_input_command(&mut self) -> String {
    // Clear status bar
    nc::wmove(self.statusbar, 0, 0);
    nc::wclrtoeol(self.statusbar);

    // Add new empty input
    self.statusbar_input.push(String::new());

    // Number of inputs in vector
    let n_inputs = self.statusbar_input.len();
    let mut cur_input = n_inputs - 1;

    // While Enter was not pressed
    // FIXME: this blocks the rest of the UI
    loop {
      let color = get_color(COLOR_PAIR_DEFAULT);
      nc::wattron(self.statusbar, color);
      nc::mvwprintw(
        self.statusbar,
        0,
        0,
        format!(":{}", self.statusbar_input.last().unwrap().as_str()).as_str(),
      );
      nc::wclrtoeol(self.statusbar);
      nc::wattroff(self.statusbar, color);
      nc::wrefresh(self.statusbar);

      let ch = nc::getch() as i32;

      // If Enter is preseed
      if ch == '\n' as i32 {
        return self.statusbar_input.last().unwrap().clone();
      }
      // If Backspace is pressed
      else if ch == KEY_BACKSPACE {
        let mut new_len = 0;
        if let Some(last_elem) = self.statusbar_input.get_mut(n_inputs - 1) {
          // Remove last character from string
          last_elem.pop();
          new_len = last_elem.len();
        }

        // Clear statusbar
        nc::wmove(self.statusbar, 0, new_len as i32);
        nc::wclrtoeol(self.statusbar);
        nc::wrefresh(self.statusbar);
      }
      // If Up is pressed
      else if ch == nc::KEY_UP {
        if cur_input > 0 {
          cur_input -= 1;
        }

        // Copy content of previous input to last input
        if cur_input < n_inputs - 1 {
          let cmd = self.statusbar_input[cur_input].clone();
          if let Some(last_elem) = self.statusbar_input.get_mut(n_inputs - 1) {
            *last_elem = cmd;
          }
        }
        // If back to last input: clear string
        else if cur_input == n_inputs - 1 {
          if let Some(last_elem) = self.statusbar_input.get_mut(n_inputs - 1) {
            last_elem.clear();
          }
        }
      }
      // If Down is pressed
      else if ch == nc::KEY_DOWN {
        if cur_input < n_inputs - 1 {
          cur_input += 1;
        }

        // Copy content of next input to last input
        if cur_input < n_inputs - 1 {
          let cmd = self.statusbar_input[cur_input].clone();
          if let Some(last_elem) = self.statusbar_input.get_mut(n_inputs - 1) {
            *last_elem = cmd;
          }
        }
        // If back to last input: clear string
        else if cur_input == n_inputs - 1 {
          if let Some(last_elem) = self.statusbar_input.get_mut(n_inputs - 1) {
            last_elem.clear();
          }
        }
      } else if let Some(c) = char::from_u32(ch as u32) {
        self.statusbar_input[n_inputs - 1].push(c);
      }
    }
  }
}

impl Drop for View {
  fn drop(&mut self) {
    destroy_win(self.header);
    destroy_win(self.state);
    destroy_win(self.main_win);
    destroy_win(self.progressbar);
    destroy_win(self.statusbar);
    deinit_ncurses();
  }
}
