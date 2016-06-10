extern crate nom;

use std::fs::File;
use std::path::PathBuf;
use std::io;
use std::io::Read;
use nom::*;

#[derive(Debug)]
pub enum ParserError {
  Io(io::Error),
  Parse(u32),
}

// Example: quit, do_something
fn is_action_name_char(ch: char) -> bool {
  // alphabetic and underscore
  is_alphabetic(ch as u8) || ch == '_'
}

fn is_line_ending_s(ch: char) -> bool {
  ch == '\r' || ch == '\n'
}

named!(line_ending_s<&str,&str>, is_a_s!("\r\n"));

named!(comment<&str,&str>,
  chain!(
    tag_s!("#") ~
    take_till_s!(is_line_ending_s),
    || ""
  )
);

named!(line_ending_or_comment<&str,&str>,
  chain!(
    opt!(comment) ~
    line_ending_s,
  || ""
  )
);

named!(ignored_line<&str,&str>,
  alt!(multispace | chain!(opt!(space) ~ line_ending_or_comment, || ""))
);

named!(def_key<&str,&str>,
  chain!(
    tag_s!("def_key") ~
    space ~
    tag_s!("\"") ~
    val: take_until_s!("\"") ~
    tag_s!("\"") ~
    opt!(space) ~
    line_ending_or_comment,
    || val
  )
);

named!(action_name<&str,&str>,
  take_while_s!(is_action_name_char)
);

// Example:
//   some_action
named!(action<&str,&str>,
  chain!(
    space ~
    val: action_name ~
    alt!(
      chain!(opt!(space) ~ opt!(line_ending_or_comment), || "") |
      eof
    ),
    || val
  )
);

// Example:
//   some_action
//   some_other_action
named!(actions_aggregator<&str, Vec<&str> >, many1!(action));

// Example:
// def_key "f"
//   some_action
//   some_other_action
named!(key_actions<&str,(&str,Vec<&str>)>,
  chain!(
    many0!(ignored_line) ~
    key: def_key ~
    actions: actions_aggregator ~
    many0!(ignored_line),
    || {(key, actions)}
  )
);

// Example:
// def_key "f"
//   some_action
//   some_other_action
//
// def_key "j"
//   do_something
named!(key_actions_aggregator<&str, Vec<(&str,Vec<&str>)> >, many0!(key_actions));

pub fn parse_bindings_configuration(path: &PathBuf)
                                    -> Result<Vec<(String, Vec<String>)>, ParserError> {
  let mut f = try!(File::open(path).map_err(ParserError::Io));
  let mut s = String::default();
  try!(f.read_to_string(&mut s).map_err(ParserError::Io));

  let data = key_actions_aggregator(s.as_str());
  match data {
    IResult::Done(_, o) => {
      let res = o.iter()
        .map(|ref val| {
          (String::from(val.0),
           val.1
            .iter()
            .map(|act| String::from(*act))
            .collect::<Vec<String>>())
        })
        .collect::<Vec<(String, Vec<String>)>>();
      Ok(res)
    }
    _ => Err(ParserError::Parse(0)),
  }
}

#[test]
fn parse_def_key() {
  let file = "def_key \"k\"
  scroll_up";
  let file_remaining = "  scroll_up";

  let def_key_res = def_key(file);
  assert_eq!(def_key_res, IResult::Done(file_remaining, "k"));
}

#[test]
fn parse_action() {
  let file = "  scroll_up
  scroll_down";
  let file_remaining = "  scroll_down";

  let action_res = action(file);
  assert_eq!(action_res, IResult::Done(file_remaining, "scroll_up"));
}

#[test]
fn parse_key_action() {
  let file = "def_key \"k\"
  scroll_up
def_key \"j\"
  scroll_down";
  let file_remaining = "def_key \"j\"
  scroll_down";

  let action_res = key_actions(file);
  assert_eq!(action_res, IResult::Done(file_remaining, ("k", vec!["scroll_up"])));
}

#[test]
fn parse_key_actions() {
  let file = "def_key \"k\"
  scroll_up
  scroll_down
def_key \"j\"
  scroll_down";
  let file_remaining = "def_key \"j\"
  scroll_down";

  let actions_res = key_actions(file);
  assert_eq!(actions_res, IResult::Done(file_remaining, ("k", vec!["scroll_up", "scroll_down"])));
}

#[test]
fn parse_multi_key_actions() {
  let file = "def_key \"k\"
  scroll_up
  scroll_down

def_key \"j\"
  scroll_down
";
  let file_remaining = "";

  let actions_res = key_actions_aggregator(file);
  assert_eq!(actions_res, IResult::Done(file_remaining, vec![("k", vec!["scroll_up", "scroll_down"]),
                                                             ("j", vec!["scroll_down"])]));
}

#[test]
fn parse_comment() {
  let files = ["# This is a comment", "### This is a title ###"];
  let file_remaining = "";

  for f in files.into_iter() {
    let comment_res = comment(f);
    assert_eq!(comment_res, IResult::Done(file_remaining, ""));
  }
}

#[test]
fn parse_ignored_line() {
  let files = ["# This is a comment
",
               "### This is a title ###
"];
  let file_remaining = "";

  for f in files.into_iter() {
    let comment_res = ignored_line(f);
    assert_eq!(comment_res, IResult::Done(file_remaining, ""));
  }
}

#[test]
fn parse_multi_key_action_with_comments() {
  let file = "# Map k
def_key \"k\"
  scroll_up
  scroll_down

# Map j
def_key \"j\"
  scroll_down

# Some comment
";
  let file_remaining = "";

  let actions_res = key_actions_aggregator(file);
  assert_eq!(actions_res, IResult::Done(file_remaining, vec![("k", vec!["scroll_up", "scroll_down"]),
                                                             ("j", vec!["scroll_down"])]));
}
