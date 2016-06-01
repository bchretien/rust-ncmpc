extern crate ncurses;

use ncurses as nc;

use std::cmp;
use std::iter;

pub struct View {
}

impl View
{
    pub fn new() -> View
    {
        View {}
    }

    pub fn init(&mut self)
    {
        // Set locale for unicode support.
        let locale_conf = nc::LcCategory::all;
        nc::setlocale(locale_conf, "en_US.UTF-8");

        // Start ncurses.
        nc::initscr();

        // Allow for extended keyboard (like F1).
        nc::keypad(nc::stdscr, true);
        nc::noecho();

        // Set timeout.
        nc::timeout(0);

        // Enable mouse events.
        nc::mousemask(nc::ALL_MOUSE_EVENTS as u64, None);

        // Get the screen bounds.
        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        nc::clear();
    }

    // TODO: data should not be mutable
    pub fn set_playlist(&mut self, desc: &[(String, u32)], data: &mut[&mut[String]])
    {
        // Get the screen bounds.
        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        // Header:
        // Move to proper line
        nc::mv(0, 0);
        // Clear line.
        nc::clrtoeol();
        let mut x = 0;
        for col in desc {
            nc::mv(0, x-1);
            nc::clrtoeol();
            nc::mvprintw(0, x, &format!("{}", col.0));
            x += 1 + col.1 as i32;
        }
        // Separator
        nc::mv(1, 0);
        nc::clrtoeol();
        let sep = iter::repeat('─').take(max_x as usize).collect::<String>();
        nc::mvprintw(1, 0, &sep);

        // Playlist data
        let playlist_start_row = 2;
        let playlist_max_row = max_y-3;
        for y in 0..cmp::min(playlist_max_row-playlist_start_row, data.len() as i32) {
            // For each column
            x = 0;
            for i in 0..desc.len() {
                nc::mv(playlist_start_row+y, x-1);
                nc::clrtoeol();
                // Print message.
                nc::mvprintw(playlist_start_row+y, x, &format!("{}", data[y as usize][i as usize]));
                x += 1 + desc[i].1 as i32;
            }
        }
        nc::mvprintw(playlist_max_row, 0, &sep);
    }

    pub fn set_playing_line(&mut self, msg: &str)
    {
        // Move to line above bottom line.
        nc::mv(nc::LINES-2, 0);
        // Clear line.
        nc::clrtoeol();
        // Print message.
        nc::mvprintw(nc::LINES-2, 0, msg);
    }

    pub fn set_debug_prompt(&mut self, msg: &str)
    {
        // Move to bottom line.
        nc::mv(nc::LINES-1, 0);
        // Clear line.
        nc::clrtoeol();
        // Print message.
        nc::mvprintw(nc::LINES-1, 0, &format!("[Debug] {}", msg));
    }

    pub fn exit(&mut self)
    {
        // Terminate ncurses.
        nc::endwin();
    }
}
