//! MPD ncurses client for Rust

extern crate ncurses;
extern crate mpd;

pub mod config;
pub mod controller;
pub mod model;
pub mod util;
pub mod view;

pub use config::{KeyConfig,ColorConfig,Config};
pub use controller::{Controller};
pub use model::{Model};
pub use view::{View};
