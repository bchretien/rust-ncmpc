extern crate ncurses;
use ncurses as nc;

use std::char;
use std::collections::HashMap;

use config::*;
use model::*;

pub type ControllerCallbacks<'m> = HashMap<i32, Box<FnMut(&mut SharedModel<'m>) + 'm>>;

pub struct Controller<'c, 'm: 'c> {
  model: &'c mut SharedModel<'m>,
  config: &'c Config,
  callbacks: ControllerCallbacks<'m>,
}

impl<'c, 'm> Controller<'c, 'm> {
  pub fn new(model: &'c mut SharedModel<'m>, config: &'c Config) -> Controller<'c, 'm> {
    {
      let mut m = model.lock().unwrap();
      m.init();
    }

    // Set callbacks
    let mut callbacks = ControllerCallbacks::new();

    // Clear the playlist
    callbacks.insert(config.keys.clear.keycode(), Box::new(playlist_clear));
    // Pause
    callbacks.insert(config.keys.play_pause.keycode(), Box::new(playlist_pause));
    // Stop
    callbacks.insert(config.keys.stop.keycode(), Box::new(playlist_stop));
    // Previous song
    callbacks.insert(config.keys.previous_song.keycode(), Box::new(playlist_previous));
    // Next song
    callbacks.insert(config.keys.next_song.keycode(), Box::new(playlist_next));

    Controller {
      model: model,
      config: config,
      callbacks: callbacks,
    }
  }

  pub fn process_input(&mut self) -> bool {
    // Get user input
    let ch = nc::getch();

    // quit
    if ch != self.config.keys.quit.keycode() {
      // no key pressed
      if ch == -1 {
        // Do nothing
      }
      // Registered callbacks
      else if let Some(f) = self.callbacks.get_mut(&ch) {
        f(self.model);
      }
      // TODO: debug only
      else {
        let mut model = self.model.lock().unwrap();
        let c = char::from_u32(ch as u32);
        if c.is_some() {
          model.display_message(&format!("Pressed unmapped '{}'", c.unwrap()));
        } else {
          model.display_message(&format!("Pressed unmapped key (code = {})", ch));
        }
      }
      return false;
    } else {
      return true;
    }
  }
}
