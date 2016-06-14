/// Color type.
pub type Color = i16;

/// Maximum number of columns.
pub static MAX_NUM_COLUMNS: usize = 10;

// Individual color handles.
pub static COLOR_DEFAULT: Color = -1;
pub static COLOR_BACKGROUND: Color = 101;
pub static COLOR_FOREGROUND: Color = 102;
pub static COLOR_BAR_START: Color = 103;
pub static COLOR_BAR_END: Color = 104;

// Color pairs; foreground && background.
// TODO: automate this once const fn is available
pub static COLOR_PAIR_DEFAULT: Color = 1;
pub static COLOR_PAIR_HEADER: Color = 2;
pub static COLOR_PAIR_ARTIST: Color = 3;
pub static COLOR_PAIR_PROGRESSBAR: Color = 4;
pub static COLOR_PAIR_PROGRESSBAR_ELAPSED: Color = 5;
pub static COLOR_PAIR_STATUSBAR: Color = 6;
pub static COLOR_PAIR_VOLUME: Color = 7;
pub static COLOR_PAIR_DEBUG: Color = 8;
pub static COLOR_PAIR_STATE_LINE: Color = 9;
pub static COLOR_PAIR_STATE_FLAGS: Color = 10;
pub static COLOR_PAIR_TRACK: Color = 11;

// TODO: find why MAX_NUM_COLUMNS cannot be used here
pub static COLOR_PAIR_COLUMNS: [Color; 10] = [20, 21, 22, 23, 24, 25, 26, 27, 28, 29];

// ctrl-?
pub static KEY_CTRL_A: i32 = 1;
pub static KEY_CTRL_LEFTBRACKET: i32 = 27;
pub static KEY_CTRL_BACKSLASH: i32 = 28;
pub static KEY_CTRL_RIGHTBRACKET: i32 = 29;
pub static KEY_CTRL_CARET: i32 = 30;
pub static KEY_CTRL_UNDERSCORE: i32 = 31;
pub static KEY_ESCAPE: i32 = 27;
pub static KEY_TAB: i32 = 9;
pub static KEY_BACKSPACE: i32 = 127;
