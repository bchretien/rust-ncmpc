extern crate ncmpc;

use std::path::PathBuf;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

fn get_config_path(name: &str) -> PathBuf {
  let mut path = PathBuf::from(file!());
  path.pop();
  path.push(name);
  path
}

fn reset_env_vars() {
  env::remove_var("MPD_HOST");
  env::remove_var("MPD_PORT");
}


/// Test used to compare our default configuration with ncmpcpp's default
/// configuration.
#[test]
fn load_default_config() {
  reset_env_vars();

  use ncmpc::{ConfigLoader, ParamConfig};
  let config_loader = ConfigLoader::new();
  let config_path = get_config_path("default_config");
  let config = config_loader.load(Some(config_path));

  let params = ParamConfig::new();
  assert_eq!(config.params, params);
}

/// Test used to check custom configurations.
#[test]
fn load_custom_config() {
  reset_env_vars();

  use ncmpc::{ConfigLoader, ParamConfig};
  let config_loader = ConfigLoader::new();
  let config_path = get_config_path("custom_config");
  let config = config_loader.load(Some(config_path));

  let mut params = ParamConfig::new();
  params.mpd_port = 7700;
  assert_eq!(config.params, params);
}

/// Test MPD socket address.
#[test]
fn mpd_socket_addr() {
  use ncmpc::{ConfigLoader, ParamConfig};
  let config_loader = ConfigLoader::new();
  let config_path = get_config_path("custom_config");
  let config = config_loader.load(Some(config_path));

  let params = ParamConfig::new();
  let addr: SocketAddr = "127.0.0.1:7700".parse().unwrap();
  assert_eq!(config.socket_addr(), addr);
}
