extern crate getopts;

use getopts::Options;
use std::env;
use std::path::PathBuf;

use config::{Config, ConfigLoader};

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} FILE [options]...", program);
  print!("{}", opts.usage(&brief));
}

fn print_version() {
  const version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
  println!("rust-ncmpc: {}", version.unwrap_or("0.0.1"));
}

/// Process command-line options, and return the config.
pub fn process_cli(args: &Vec<String>) -> Option<Config> {
  let mut opts = Options::new();
  opts.optopt("h", "host", "connect to server at host", "arg (=localhost)");
  opts.optopt("p", "port", "connect to server at port", "arg (=6600)");
  opts.optopt("c", "config", "specify configuration file", "arg (=~/.config/ncmpcpp/config)");
  opts.optopt("b", "bindings", "specify bindings file", "arg (=~/.config/ncmpcpp/bindings)");
  opts.optflag("?", "help", "show help message");
  opts.optflag("v", "version", "display version information");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => panic!(f.to_string()),
  };

  if matches.opt_present("v") {
    print_version();
    return None;
  }

  if matches.opt_present("?") {
    print_usage(&args[0].clone(), opts);
    return None;
  }

  let opt_config = match matches.opt_str("c") {
    Some(p) => Some(PathBuf::from(p)),
    None => None,
  };
  let opt_bindings = match matches.opt_str("b") {
    Some(p) => Some(PathBuf::from(p)),
    None => None,
  };

  // Load config.
  let config_loader = ConfigLoader::new();
  let mut config = config_loader.load(opt_config, opt_bindings);

  if matches.opt_present("h") {
    config.params.mpd_host = matches.opt_str("h").unwrap();
  }

  if matches.opt_present("p") {
    let port = matches.opt_str("p").unwrap().parse::<u16>();
    if port.is_ok() {
      config.params.mpd_port = port.unwrap();
    }
  }

  return Some(config);
}
