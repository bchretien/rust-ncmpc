//! MPD ncurses client for Rust
#[macro_use]
extern crate nom;

#[macro_use]
pub mod util;

extern crate ncurses;
extern crate mpd;
extern crate time;
extern crate ini;
extern crate getopts;

pub mod action;
pub mod cli;
pub mod config;
pub mod constants;
pub mod controller;
pub mod format;
pub mod help;
pub mod model;
pub mod parser;
pub mod view;

pub use cli::process_cli;
pub use config::{ConfigLoader, ParamConfig};
pub use constants::Color;
pub use controller::{ControlQuery, Controller};
pub use format::{Column, SongProperty};
pub use model::Model;
pub use parser::parse_bindings_configuration;
pub use view::View;
