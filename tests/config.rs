extern crate ncmpc;

use std::path::PathBuf;

fn get_config_path(name: &str) -> PathBuf {
  let mut path = PathBuf::from(file!());
  path.pop();
  path.push(name);
  path
}


#[test]
/// Test used to compare our default configuration with ncmpcpp's default
/// configuration.
fn load_default_config() {
  use ncmpc::{ConfigLoader, ParamConfig};
  let config_loader = ConfigLoader::new();
  let config_path = get_config_path("default_config");
  let config = config_loader.load(Some(config_path));

  let params = ParamConfig::new();
  assert_eq!(config.params, params);
}
