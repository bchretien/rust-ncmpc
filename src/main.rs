extern crate ncmpc;

use std::sync::{Arc, Mutex};


fn main() {
  use ncmpc::{ConfigLoader, Controller, Model, View};

  let config_loader = ConfigLoader::new();

  // Load config.
  let config = config_loader.load(None);

  // Instantiate view.
  let mut view = View::new(&config.colors);

  // Instantiate model.
  let mut model = Arc::new(Mutex::new(Model::new(&mut view, &config)));
  let shared_model = model.clone();

  // Instantiate controller.
  let mut controller = Controller::new(&mut model, &config);

  // Start the TUI loop (automatic refresh).
  loop {
    // Process user input.
    if controller.process_input() {
      break;
    }

    // Refresh TUI.
    {
      let mut m = shared_model.lock().unwrap();
      m.display_playlist();
      m.display_play_bar();
      m.display_now_playing();
    }
  }
}
