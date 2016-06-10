extern crate ncmpc;

use std::path::PathBuf;

fn before_each() {}

fn after_each() {}

fn get_bindings_path(name: &str) -> PathBuf {
  let mut path = PathBuf::from(file!());
  path.pop();
  path.push(name);
  path
}

#[test]
fn load_default_bindings() {
  use ncmpc::parse_bindings_configuration;

  before_each();
  let res = parse_bindings_configuration(&get_bindings_path("default_bindings"));
  assert!(res.is_ok());
  let data = res.unwrap();
  assert_eq!(data[0], (String::from("mouse"), vec![String::from("mouse_event")]));
  assert_eq!(data[2],
             (String::from("shift-up"),
              vec![String::from("select_item"), String::from("scroll_up")]));
  assert_eq!(data.last().unwrap(), &(String::from("q"), vec![String::from("quit")]));
  after_each();
}
