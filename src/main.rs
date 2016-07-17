extern crate ncmpc;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::env;


fn main() {
  use ncmpc::{ControlQuery, Controller, Model, View, process_cli};

  // Process CLI options and return config.
  let args: Vec<String> = env::args().collect();
  let opt_config = process_cli(&args);
  let config = match opt_config {
    Some(o) => o,
    None => return,
  };

  // Instantiate view.
  let mut view = View::new(&config);

  // Instantiate model.
  let mut model = Arc::new(Mutex::new(Model::new(&mut view, &config)));
  let shared_model = model.clone();

  // Instantiate controller.
  let mut controller = Controller::new(&mut model, &config);

  // Start the TUI loop (automatic refresh).
  loop {
    // Process user input, and exit if required.
    match controller.process_input() {
      ControlQuery::Exit => break,
      ControlQuery::Nothing => sleep(Duration::from_millis(50)),
      ControlQuery::Command => {}
    }

    // Refresh TUI.
    {
      let mut m = shared_model.lock().unwrap();
      m.take_snapshot();
      m.update_header();
      m.update_stateline();
      m.update_main_window();
      m.update_progressbar();
      m.update_statusbar();
    }
  }
}
