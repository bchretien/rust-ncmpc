extern crate ncmpc;
extern crate crossbeam;

extern crate ncurses;
use ncurses as nc;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, TryRecvError};


fn main() {
    use ncmpc::{Config,Controller,Model,View};

    // Load config.
    let config = Config::new();

    // Instantiate view.
    let mut view = View::new();

    // Instantiate model.
    let mut model = Arc::new(Mutex::new(Model::new(&mut view, &config)));
    let shared_model = model.clone();

    // Instantiate controller.
    let mut controller = Controller::new(&mut model, &config);

    // Start the TUI loop (automatic refresh).
    let (tx, rx) = mpsc::channel::<bool>();
    loop {
        // Process user input.
        if controller.process_input()
        {
            break;
        }

        // Refresh TUI.
        {
            let mut m = shared_model.lock().unwrap();
            m.display_playlist();
            m.display_now_playing();
        }
    }

    {
        let mut m = shared_model.lock().unwrap();
        m.deinit();
    }
}
