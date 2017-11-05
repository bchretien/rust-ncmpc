/// Color type.
pub type Color = i16;

/// Maximum number of columns.
pub const MAX_NUM_COLUMNS: usize = 10;

// Individual color handles.
pub const COLOR_DEFAULT: Color = -1;
pub const COLOR_BACKGROUND: Color = 101;
pub const COLOR_FOREGROUND: Color = 102;
pub const COLOR_BAR_START: Color = 103;
pub const COLOR_BAR_END: Color = 104;

// Color pairs
pub const COLOR_PAIR_DEFAULT: Color = 0;
pub const COLOR_PAIR_BLACK: Color = 1;
pub const COLOR_PAIR_RED: Color = 2;
pub const COLOR_PAIR_GREEN: Color = 3;
pub const COLOR_PAIR_YELLOW: Color = 4;
pub const COLOR_PAIR_BLUE: Color = 5;
pub const COLOR_PAIR_MAGENTA: Color = 6;
pub const COLOR_PAIR_CYAN: Color = 7;
pub const COLOR_PAIR_WHITE: Color = 8;

// TODO: automate this once const fn is available
pub const COLOR_PAIR_HEADER: Color = 210;
pub const COLOR_PAIR_PROGRESSBAR: Color = 211;
pub const COLOR_PAIR_PROGRESSBAR_ELAPSED: Color = 212;
pub const COLOR_PAIR_STATUSBAR: Color = 213;
pub const COLOR_PAIR_VOLUME: Color = 214;
pub const COLOR_PAIR_DEBUG: Color = 215;
pub const COLOR_PAIR_STATE_LINE: Color = 216;
pub const COLOR_PAIR_STATE_FLAGS: Color = 217;
pub const COLOR_PAIR_TRACK: Color = 218;
pub const COLOR_PAIR_WINDOW_BORDER: Color = 219;

pub const COLOR_PAIR_COLUMNS: [Color; MAX_NUM_COLUMNS] = [30, 31, 32, 33, 34, 35, 36, 37, 38, 39];

// ctrl-?
pub const KEY_CTRL_A: i32 = 1;
pub const KEY_CTRL_LEFTBRACKET: i32 = 27;
pub const KEY_CTRL_BACKSLASH: i32 = 28;
pub const KEY_CTRL_RIGHTBRACKET: i32 = 29;
pub const KEY_CTRL_CARET: i32 = 30;
pub const KEY_CTRL_UNDERSCORE: i32 = 31;
pub const KEY_ESCAPE: i32 = 27;
pub const KEY_TAB: i32 = 9;
pub const KEY_BACKSPACE: i32 = 127;
