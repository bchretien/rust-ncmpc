extern crate ncurses;

use ncurses as nc;

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
        // Start ncurses.
        nc::initscr();

        // Allow for extended keyboard (like F1).
        nc::keypad(nc::stdscr, true);
        nc::noecho();

        // Enable mouse events.
        nc::mousemask(nc::ALL_MOUSE_EVENTS as u64, None);

        // Get the screen bounds.
        let mut max_x = 0;
        let mut max_y = 0;
        nc::getmaxyx(nc::stdscr, &mut max_y, &mut max_x);

        // Print intro + controls
        nc::clear();
        nc::mvprintw(2, 4, "Welcome to Rust MPD client for ncurses");
        nc::mvprintw(9, 4, "Press any key to begin...");
        nc::refresh();
        nc::getch();
        nc::clear();
    }

    pub fn playlist_play(&mut self) {
        // TODO
    }

    pub fn playlist_stop(&mut self) {
        // TODO
    }

    pub fn exit(&mut self)
    {
        // Terminate ncurses.
        nc::endwin();
    }
}
