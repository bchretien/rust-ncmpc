extern crate ncurses;
use ncurses as nc;

use std::char;
use std::collections::HashMap;

use config::*;
use model::*;

pub enum ControlQuery {
  /// Some query was made.
  Command,
  /// No query was made.
  Nothing,
  /// Exit query.
  Exit,
}

pub type ControllerCallbacks<'m> = HashMap<i32, Vec<Action<'m>>>;

pub struct Controller<'c, 'm: 'c> {
  model: &'c mut SharedModel<'m>,
  config: &'c Config,
  callbacks: ControllerCallbacks<'m>,
  quit_keycodes: Vec<i32>,
}

macro_rules! register_callback {
  // If the keycode is part of the configuration
  ($callbacks: ident, $config: ident, $action:ident, $callback: ident) => {
    {
      for key in &$config.keys.$action {
        $callbacks.insert(key.keycode(), vec![Action::new($callback)]);
      }
    }
  };
  // For special keycodes
  ($callbacks: ident, $key:expr, $callback: ident) => {
    {
      $callbacks.insert($key, vec![Action::new($callback)]);
    }
  };
  // For custom actions
  ($callbacks: ident, $map: ident, $key:expr, actions => $actions: ident) => {
    {
      let user_actions = $actions.iter()
        .filter_map(|ref name| $map.get(name.as_str()))
        .cloned()
        .collect::<Vec<Action<'m>>>();
      $callbacks.insert($key.clone(), user_actions);
    }
  };
}

impl<'c, 'm> Controller<'c, 'm> {
  pub fn new(model: &'c mut SharedModel<'m>, config: &'c Config) -> Controller<'c, 'm> {
    // Set callbacks
    let mut callbacks = ControllerCallbacks::new();

    // Clear the playlist
    register_callback!(callbacks, config, clear, playlist_clear);
    // Delete selected items
    register_callback!(callbacks, config, delete, playlist_delete_items);
    // Pause
    register_callback!(callbacks, config, play_pause, playlist_pause);
    // Stop
    register_callback!(callbacks, config, stop, playlist_stop);
    // Previous song
    register_callback!(callbacks, config, previous_song, playlist_previous);
    // Next song
    register_callback!(callbacks, config, next_song, playlist_next);
    // Increase volume
    register_callback!(callbacks, config, volume_up, volume_up);
    // Decrease volume
    register_callback!(callbacks, config, volume_down, volume_down);
    // Press enter
    register_callback!(callbacks, config, press_enter, play_selected);
    // Scroll down
    register_callback!(callbacks, config, scroll_down, scroll_down);
    // Scroll up
    register_callback!(callbacks, config, scroll_up, scroll_up);
    // Show help
    register_callback!(callbacks, config, show_help, show_help);
    // Show playlist
    register_callback!(callbacks, config, show_playlist, show_playlist);
    // Toggle bitrate visibility
    register_callback!(callbacks, config, toggle_bitrate_visibility, toggle_bitrate_visibility);
    // Toggle random
    register_callback!(callbacks, config, toggle_random, toggle_random);
    // Toggle repeat
    register_callback!(callbacks, config, toggle_repeat, toggle_repeat);
    // Mouse support
    register_callback!(callbacks, nc::KEY_MOUSE, process_mouse);
    // Resize windows
    register_callback!(callbacks, nc::KEY_RESIZE, resize_windows);

    // Register custom user actions (possibly overriding defaults).
    let action_map = get_action_map();
    for (keycode, actions) in &config.keys.custom {
      register_callback!(callbacks, action_map, keycode, actions => actions);
    }

    let quit_keycodes = config.keys.quit.iter().map(|&key| key.keycode()).collect::<Vec<i32>>();

    Controller {
      model: model,
      config: config,
      callbacks: callbacks,
      quit_keycodes: quit_keycodes,
    }
  }

  pub fn process_input(&mut self) -> ControlQuery {
    // Get user input
    let ch = nc::getch();

    // Quit check
    if !self.quit_keycodes.contains(&ch) {
      // No key pressed
      if ch == -1 {
        // Do nothing
        return ControlQuery::Nothing;
      }
      // Registered callbacks
      else if let Some(actions) = self.callbacks.get_mut(&ch) {
        for action in actions {
          action.execute(self.model);
        }
      }
      // TODO: debug only
      else {
        let mut model = self.model.lock().unwrap();
        model.update_message(&format!("Pressed unmapped '{}' (keycode = {})", nc::keyname(ch), ch));
      }
      return ControlQuery::Command;
    } else {
      return ControlQuery::Exit;
    }
  }
}
