extern crate ncurses;

use ncurses as nc;

use std::cmp;
use std::iter;

use constants::*;
use config::{ColorConfig};

pub struct View {
    colors: ColorConfig,
    playlist_row: nc::WINDOW,
    parameters_row: nc::WINDOW,
    main_win: nc::WINDOW,
    play_bar: nc::WINDOW,
    bottom_row: nc::WINDOW,
    debug_row: nc::WINDOW,
}

fn init_colors(colors: &ColorConfig) {
    nc::start_color();

    // Background transparency
    let mut color_bg = nc::COLOR_BLACK;
    let mut color_fg = nc::COLOR_WHITE;
    if nc::use_default_colors() == nc::OK
    {
        color_fg = -1;
        color_bg = -1;
    }

    nc::init_pair(COLOR_PAIR_DEFAULT, color_fg, color_bg);
    nc::init_pair(COLOR_PAIR_ARTIST, nc::COLOR_YELLOW, color_bg);
    nc::init_pair(COLOR_PAIR_HEADER, nc::COLOR_WHITE, color_bg);
    nc::init_pair(COLOR_PAIR_PROGRESSBAR, colors.progressbar, color_bg);
    nc::init_pair(COLOR_PAIR_PROGRESSBAR_ELAPSED, colors.progressbar_elapsed, color_bg);
    nc::init_pair(COLOR_PAIR_BOTTOM, nc::COLOR_BLUE, color_bg);
    nc::init_pair(COLOR_PAIR_DEBUG, nc::COLOR_GREEN, color_bg);
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
    // nc::start_color();
    nc::cbreak();

    // Allow for extended keyboard (like F1).
    nc::keypad(nc::stdscr, true);
    nc::noecho();

    // Set timeout.
    nc::timeout(0);

    // Enable mouse events.
    nc::mousemask(nc::ALL_MOUSE_EVENTS as u64, None);

    nc::clear();
}

// TODO: check 32/64 bit attr_t
fn get_color(c: i16) -> i32 {
    return nc::COLOR_PAIR(c) as i32
}

fn bold() -> i32 {
    return nc::A_BOLD() as i32
}

fn deinit_ncurses() {
    // Terminate ncurses.
    nc::endwin();
}

fn destroy_win(win: nc::WINDOW)
{
  let ch = ' ' as nc::chtype;
  nc::wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
  nc::wrefresh(win);
  nc::delwin(win);
}

impl View
{
    pub fn new(c: &ColorConfig) -> View
    {
        let colors = c.clone();

        init_ncurses(&colors);

        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        let view = View {
            colors: colors,
            playlist_row: nc::newwin(1, max_x, 0, 0),
            parameters_row: nc::newwin(1, max_x, 1, 0),
            main_win: nc::newwin(max_y-5, max_x, 2, 0),
            play_bar: nc::newwin(1, max_x, max_y-3, 0),
            bottom_row: nc::newwin(1, max_x, max_y-2, 0),
            debug_row: nc::newwin(1, max_x, max_y-1, 0),
        };
        nc::wrefresh(view.playlist_row);
        nc::wrefresh(view.parameters_row);
        nc::wrefresh(view.main_win);
        nc::wrefresh(view.play_bar);
        nc::wrefresh(view.bottom_row);
        nc::wrefresh(view.debug_row);
        nc::keypad(view.main_win, true);

        // Set colors
        nc::wbkgd(view.playlist_row, nc::COLOR_PAIR(COLOR_PAIR_ARTIST) as nc::chtype);
        nc::wbkgd(view.parameters_row, nc::COLOR_PAIR(COLOR_PAIR_DEFAULT) as nc::chtype);
        nc::wbkgd(view.debug_row, nc::COLOR_PAIR(COLOR_PAIR_DEBUG) as nc::chtype);

        return view;
    }

    // TODO: data should not be mutable
    pub fn set_playlist(&mut self, desc: &[(String, u32)], data: &mut[&mut[String]])
    {
        // Get the screen bounds.
        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        let mut color = get_color(COLOR_PAIR_HEADER);

        nc::wattron(self.main_win, bold());
        nc::wattron(self.main_win, color);

        // Header
        let mut x = 0;
        for col in desc {
            nc::wmove(self.main_win, 0, x-1);
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
        let playlist_start_row = 2;
        let playlist_max_row = max_y-3;
        let height = cmp::min(playlist_max_row, data.len() as i32);

        color = get_color(COLOR_PAIR_ARTIST);
        nc::wattron(self.main_win, color);
        for y in 0..cmp::min(playlist_max_row-playlist_start_row, data.len() as i32) {
            // For each column
            x = 0;
            for i in 0..desc.len() {
                nc::wmove(self.main_win, playlist_start_row+y, x-1);
                nc::wclrtoeol(self.main_win);
                nc::mvwprintw(self.main_win, playlist_start_row+y, x, &format!("{}", data[y as usize][i as usize]));
                x += 1 + desc[i].1 as i32;
            }
        }
        nc::wattroff(self.main_win, color);

        nc::wrefresh(self.main_win);
    }

    pub fn set_play_bar(&mut self, pct: f32)
    {
        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        let tip_x: i32 = (pct/100. * (max_x as f32)) as i32;

        // Start of the bar
        let len_start: usize = tip_x as usize;
        let sep = iter::repeat('─').take(len_start).collect::<String>();
        let mut color = get_color(COLOR_PAIR_PROGRESSBAR_ELAPSED);
        nc::wattron(self.play_bar, color);
        nc::mvwprintw(self.play_bar, 0, 0, &sep);

        // Tip of the bar
        let tip = "╼";
        nc::mvwprintw(self.play_bar, 0, tip_x, &tip);
        nc::wattroff(self.play_bar, color);

        // End of the bar
        let len_end: usize = (max_x - tip_x) as usize;
        let sep = iter::repeat('─').take(len_end).collect::<String>();
        color = get_color(COLOR_PAIR_PROGRESSBAR);
        nc::wattron(self.play_bar, color);
        nc::mvwprintw(self.play_bar, 0, tip_x+1, &sep);
        nc::wattroff(self.play_bar, color);

        nc::wrefresh(self.play_bar);
    }

    pub fn set_playing_line(&mut self, msg: &str)
    {
        // Clear line.
        nc::wmove(self.bottom_row, 0, 0);
        nc::wclrtoeol(self.bottom_row);
        // Print message.
        let color = get_color(COLOR_PAIR_BOTTOM);
        nc::wattron(self.bottom_row, color);
        nc::mvwprintw(self.bottom_row, 0, 0, msg);
        nc::wattroff(self.bottom_row, color);
        nc::wrefresh(self.bottom_row);
    }

    pub fn set_debug_prompt(&mut self, msg: &str)
    {
        // Clear line.
        nc::wmove(self.debug_row, 0, 0);
        nc::wclrtoeol(self.debug_row);
        // Print message.
        nc::mvwprintw(self.debug_row, 0, 0, &format!("[Debug] {}", msg));
        nc::wrefresh(self.debug_row);
    }

    fn drop(&mut self) {
        destroy_win(self.playlist_row);
        destroy_win(self.parameters_row);
        destroy_win(self.main_win);
        destroy_win(self.play_bar);
        destroy_win(self.bottom_row);
        destroy_win(self.debug_row);
        deinit_ncurses();
    }
}
