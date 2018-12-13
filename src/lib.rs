//! Clippy lints

#![cfg_attr(feature = "cargo-clippy", allow(clippy::needless_return, clippy::redundant_field_names))]

//! MPD ncurses client for Rust
extern crate nom;

#[macro_use]
pub mod util;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate getopts;
extern crate ini;
extern crate mpd;
extern crate ncurses;
extern crate time;

pub mod action;
pub mod cli;
pub mod config;
pub mod constants;
pub mod controller;
pub mod format;
pub mod help;
pub mod model;
pub mod parser;
pub mod server_info;
pub mod view;

pub use crate::cli::process_cli;
pub use crate::config::{ConfigLoader, ParamConfig};
pub use crate::constants::Color;
pub use crate::controller::{ControlQuery, Controller};
pub use crate::format::{Column, SongProperty};
pub use crate::model::Model;
pub use crate::parser::parse_bindings_configuration;
pub use crate::view::View;
