extern crate ncmpc;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;


fn main() {
  use ncmpc::{ConfigLoader, ControlQuery, Controller, Model, View};

  let config_loader = ConfigLoader::new();

  // Load config.
  let config = config_loader.load(None, None);

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
      m.update_playlist();
      m.update_progressbar();
      m.update_statusbar();
    }
  }
}
