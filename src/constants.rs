// Individual color handles.
pub static COLOR_DEFAULT: i16 = -1;
pub static COLOR_BACKGROUND: i16 = 101;
pub static COLOR_FOREGROUND: i16 = 102;
pub static COLOR_BAR_START: i16 = 103;
pub static COLOR_BAR_END: i16 = 104;

// Color pairs; foreground && background.
// TODO: automate this once const fn is available
pub static COLOR_PAIR_DEFAULT: i16 = 1;
pub static COLOR_PAIR_HEADER: i16 = 2;
pub static COLOR_PAIR_ARTIST: i16 = 3;
pub static COLOR_PAIR_PROGRESSBAR: i16 = 4;
pub static COLOR_PAIR_PROGRESSBAR_ELAPSED: i16 = 5;
pub static COLOR_PAIR_STATUSBAR: i16 = 6;
pub static COLOR_PAIR_VOLUME: i16 = 7;
pub static COLOR_PAIR_DEBUG: i16 = 8;
pub static COLOR_PAIR_STATE_LINE: i16 = 9;
pub static COLOR_PAIR_STATE_FLAGS: i16 = 10;
pub static COLOR_PAIR_TRACK: i16 = 11;

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
