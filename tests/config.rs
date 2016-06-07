extern crate ncmpc;

use std::path::PathBuf;
use std::env;

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
