use config::{Config, ControlKeys};
use ncurses as nc;

use view::{bold, get_color};

pub struct Help {
  win: nc::WINDOW,
  config: Config,
  current_row: i32,
  tab_size: i32,
  key_col_size: i32,
}

impl Help {
  pub fn new(win: nc::WINDOW, config: &Config) -> Help {
    Help {
      win: win,
      config: config.clone(),
      current_row: 0,
      tab_size: 2,
      key_col_size: 20,
    }
  }

  fn newline(&mut self) {
    self.current_row += 1;
  }

  fn section(&mut self, name: &str) {
    nc::wattron(self.win, bold());
    nc::mvwprintw(self.win, self.current_row, self.tab_size, &name);
    nc::wattroff(self.win, bold());
    self.current_row += 1;
  }

  fn print_entry(&self, name: &str, desc: &str) {
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size, &name);
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size + self.key_col_size, ": ");
    nc::mvwprintw(self.win, self.current_row, 2 * self.tab_size + self.key_col_size + 2, &desc);
  }

  fn keys(&self, keys: &ControlKeys, desc: &str) {
    let keys_s: String = keys.iter().fold(
      String::default(),
      |acc, &x| if acc.is_empty() {
        format!("{}", x)
      } else {
        acc + format!(" {}", x).as_str()
      },
    );
    self.print_entry(keys_s.as_str(), desc);
  }


  pub fn print(&mut self) {
    macro_rules! print_key(
      ($k:ident, $desc:expr) => (
        self.keys(&self.config.keys.$k, $desc);
        self.newline();
        )
      );
    macro_rules! print_text(
      ($k:expr, $desc:expr) => (
        self.print_entry($k, $desc);
        self.newline();
        )
      );

    nc::wclear(self.win);
    self.current_row = 0;

    self.newline();
    self.section("Keys - Movement");
    self.newline();
    print_key!(scroll_up, "Move cursor up");
    print_key!(scroll_down, "Move cursor down");
    print_key!(move_home, "Home");
    print_key!(move_end, "End");
    self.newline();
    print_key!(show_help, "Show help");
    print_key!(show_playlist, "Show playlist");
    self.newline();
    print_key!(show_server_info, "Show server info");

    self.newline();
    self.section("Keys - Global");
    self.newline();
    print_key!(stop, "Stop");
    print_key!(play_pause, "Pause");
    print_key!(next_song, "Next track");
    print_key!(previous_song, "Previous track");
    print_key!(
      volume_down,
      format!("Decrease volume by {}%%", self.config.params.volume_change_step).as_str()
    );
    print_key!(
      volume_up,
      format!("Increase volume by {}%%", self.config.params.volume_change_step).as_str()
    );
    self.newline();
    print_key!(toggle_repeat, "Toggle repeat mode");
    print_key!(toggle_random, "Toggle random mode");
    print_key!(toggle_bitrate_visibility, "Toggle bitrate visibility");
    self.newline();
    print_key!(quit, "Quit");

    self.newline();
    self.section("Keys - Playlist");
    self.newline();
    print_key!(press_enter, "Play selected item");
    print_key!(delete, "Delete selected item(s) from playlist");
    print_key!(clear, "Clear playlist");

    self.newline();
    self.section("Mouse - Playlist");
    self.newline();
    print_text!("Left click", "Select pointed item");
    print_text!("Right click", "Play");

    self.newline();
    self.section("List of available colors");
    self.newline();

    let mut pos = 0;
    for i in 1..232 {
      let color = get_color(i);
      nc::wattron(self.win, color);
      nc::mvwprintw(self.win, self.current_row, pos, format!("{} ", i).as_str());
      nc::wattroff(self.win, color);
      pos += 3;
    }

    nc::wrefresh(self.win);
  }
}
