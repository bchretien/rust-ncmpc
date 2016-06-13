extern crate nom;

use std::fs::File;
use std::path::PathBuf;
use std::io;
use std::io::Read;
use nom::*;

use format::Column;

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

/// Load bindings configuration from a given path.
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

fn to_width(s: &str) -> Result<(i32, bool), ParserError> {
  let is_fixed = if s.chars().last().unwrap_or(' ') == 'f' { true } else { false };
  let width = if is_fixed {
    // TODO: return error if parsing fails
    s[..s.len() - 1].parse::<i32>().unwrap_or(1)
  } else {
    s.parse::<i32>().unwrap_or(1)
  };
  return Ok((width, is_fixed));
}

// Example:
// (5f)[red]{b}
named!(column<&str,(i32, bool, &str, &str)>,
  chain!(
    opt!(space) ~
    tag_s!("(") ~
    width: map_res!(take_until_s!(")"), to_width) ~
    tag_s!(")") ~
    tag_s!("[") ~
    color: take_until_s!("]") ~
    tag_s!("]") ~
    tag_s!("{") ~
    tag: take_until_s!("}") ~
    tag_s!("}"),
    || (width.0, width.1, color, tag)
  )
);

// Example:
// (20)[]{a} (6f)[green]{NE} (50)[white]{t|f:Title}
named!(columns<&str, Vec<(i32, bool, &str, &str)> >, many1!(column));

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

#[test]
fn parse_column() {
  let file = "(20)[yellow]{a}";
  let file_remaining = "";

  let column_res = column(file);
  assert_eq!(column_res, IResult::Done(file_remaining, (20, false, "yellow", "a")));
}

#[test]
fn parse_fixed_column() {
  let file = "(5f)[red]{b}";
  let file_remaining = "";

  let column_res = column(file);
  assert_eq!(column_res, IResult::Done(file_remaining, (5, true, "red", "b")));
}

#[test]
fn parse_columns() {
  let file = "(20)[]{a} (6f)[green]{NE} (50)[white]{t|f:Title} (20)[cyan]{b} (7f)[magenta]{l}";
  let file_remaining = "";

  let columns_res = columns(file);
  assert_eq!(columns_res, IResult::Done(file_remaining, vec![(20, false, "", "a"),
                                                             (6, true, "green", "NE"),
                                                             (50, false, "white", "t|f:Title"),
                                                             (20, false, "cyan", "b"),
                                                             (7, true, "magenta", "l")]));
}
