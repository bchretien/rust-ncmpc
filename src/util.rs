use time::{get_time, Duration, Timespec};

/// Print to stderr.
#[macro_export]
macro_rules! stderr(
  ($($arg:tt)*) => {{
    use std::io::Write;
    let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
    r.expect("failed printing to stderr");
  }}
);

/// Utility structure that associates a timestamp with a value.
#[derive(Clone, Copy)]
pub struct TimedValue<T> {
  pub value: T,
  pub timestamp: Timespec,
}

impl<T: Clone> TimedValue<T> {
  pub fn new(value: T) -> TimedValue<T> {
    TimedValue::<T> {
      value: value.clone(),
      timestamp: get_time(),
    }
  }

  pub fn bump(&mut self) {
    self.timestamp = get_time();
  }
}

/// Utility structure used to define an horizontal scrolling area.
pub struct Scroller {
  /// Width of the horizontal scrolling area.
  width: usize,
  /// Current start of the scrolling area.
  pos: usize,
  /// Text to display.
  text: String,
  /// Separator.
  separator: String,
  /// Time of the last position increment.
  pos_update_time: Timespec,
  /// Number of seconds between position increments.
  dt: Duration,
  /// Temporary string.
  temp: String,
}

impl Scroller {
  pub fn new(width: usize) -> Scroller {
    Scroller {
      width: width,
      pos: 0,
      text: String::default(),
      separator: String::from(" ** "),
      pos_update_time: get_time(),
      dt: Duration::milliseconds(500),
      temp: String::with_capacity(width + 4),
    }
  }

  pub fn reset_pos(&mut self) {
    self.pos = 0;
    self.pos_update_time = get_time();
  }

  pub fn resize(&mut self, width: i32) {
    self.width = if width <= 0 { 0 } else { width as usize };
    if self.width > self.temp.capacity() {
      self.temp.reserve_exact(self.width);
    }
  }

  pub fn set_text(&mut self, text: &str) {
    self.text = String::from(text);
  }

  pub fn display(&mut self) -> &str {
    // Update pos.
    let current_t = get_time();
    if self.pos_update_time + self.dt < current_t {
      self.pos = if 1 + self.pos < self.text.len() + self.separator.len() {
        self.pos + 1
      } else {
        0
      };
      self.pos_update_time = current_t;
    }
    // Case 1: we can simply return the full text.
    if self.width >= self.text.len() {
      return &self.text;
    }
    // Case 2: the partial text does not reach the end of the text.
    else if self.pos + self.width <= self.text.len() {
      // We can simply return the full text.
      return &self.text[self.pos..self.pos + self.width];
    }
    // Case 3: we need to print both the end and the beginning of the string,
    // with a separator.
    else {
      self.temp.clear();

      // End of the text
      if self.pos < self.text.len() {
        self.temp.push_str(&self.text[self.pos..]);
      }

      // Separator
      let mut free_len: i32 = self.width as i32 - self.temp.len() as i32;
      let start_sep: i32 = if self.pos >= self.text.len() {
        self.pos as i32 - self.text.len() as i32
      } else {
        0
      };
      if free_len < self.separator.len() as i32 - start_sep as i32 {
        self.temp.push_str(&self.separator[start_sep as usize..free_len as usize]);
      } else {
        self.temp.push_str(&self.separator[start_sep as usize..]);
      }

      // Beginning of the text
      free_len = self.width as i32 - self.temp.len() as i32;
      if free_len >= 0 {
        self.temp.push_str(&self.text[..free_len as usize]);
      }
      assert!(self.temp.len() == self.width, "error in scrolled message size");
      return &self.temp;
    }
  }
}
