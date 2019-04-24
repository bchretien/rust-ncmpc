extern crate nom;

use nom::*;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

pub use nom::types::CompleteStr as cstr;

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

named!(line_ending_s<cstr, cstr>, is_a_s!("\r\n"));

named!(comment<cstr,cstr>,
  do_parse!(
    tag_s!("#") >>
    take_till_s!(is_line_ending_s) >>
    (cstr(""))
  )
);

named!(line_ending_or_comment<cstr,cstr>,
  do_parse!(
    opt!(comment) >>
    line_ending_s >>
    (cstr(""))
  )
);

named!(ignored_line<cstr,cstr>,
  alt!(multispace | do_parse!(opt!(space) >> line_ending_or_comment >> (cstr(""))))
);

named!(def_key<cstr,cstr>,
  do_parse!(
    tag_s!("def_key") >>
    space >>
    tag_s!("\"") >>
    val: take_until_s!("\"") >>
    tag_s!("\"") >>
    opt!(space) >>
    line_ending_or_comment >>
    (val)
  )
);

named!(action_name<cstr,cstr>,
  take_while_s!(is_action_name_char)
);

// Example:
//   some_action
named!(action<cstr,cstr>,
  do_parse!(
    space >>
    val: action_name >>
    alt!(
      do_parse!(opt!(space) >> opt!(line_ending_or_comment) >> (cstr(""))) |
      eof!()
    ) >>
    (val)
  )
);

// Example:
//   some_action
//   some_other_action
named!(actions_aggregator<cstr, Vec<cstr> >, many1!(action));

// Example:
// def_key "f"
//   some_action
//   some_other_action
named!(key_actions<cstr,(cstr,Vec<cstr>)>,
  do_parse!(
    many0!(ignored_line) >>
    key: def_key >>
    actions: actions_aggregator >>
    many0!(ignored_line) >>
    (key, actions)
  )
);

// Example:
// def_key "f"
//   some_action
//   some_other_action
//
// def_key "j"
//   do_something
named!(key_actions_aggregator<cstr, Vec<(cstr,Vec<cstr>)> >, many0!(key_actions));

/// Load bindings configuration from a given path.
pub fn parse_bindings_configuration(path: &PathBuf) -> Result<Vec<(String, Vec<String>)>, ParserError> {
  let mut f = r#try!(File::open(path).map_err(ParserError::Io));
  let mut s = String::default();
  r#try!(f.read_to_string(&mut s).map_err(ParserError::Io));

  let data = key_actions_aggregator(cstr(s.as_str()));
  match data {
    Ok((_, o)) => {
      let res = o
        .iter()
        .map(|ref val| {
          (
            String::from(*val.0),
            val.1.iter().map(|act| String::from(**act)).collect::<Vec<String>>(),
          )
        })
        .collect::<Vec<(String, Vec<String>)>>();
      Ok(res)
    }
    _ => Err(ParserError::Parse(0)),
  }
}

fn to_width(s: cstr) -> Result<(i32, bool), ParserError> {
  let is_fixed = s.chars().last().unwrap_or(' ') == 'f';
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
named!(column<cstr,(i32, bool, cstr, cstr)>,
  do_parse!(
    opt!(space) >>
    tag_s!("(") >>
    width: map_res!(take_until_s!(")"), to_width) >>
    tag_s!(")") >>
    tag_s!("[") >>
    color: take_until_s!("]") >>
    tag_s!("]") >>
    tag_s!("{") >>
    tag: take_until_s!("}") >>
    tag_s!("}") >>
    (width.0, width.1, color, tag)
  )
);

// Example:
// (20)[]{a} (6f)[green]{NE} (50)[white]{t|f:Title}
named!(pub get_columns_format<cstr, Vec<(i32, bool, cstr, cstr)> >, many1!(column));

#[test]
fn parse_def_key() {
  let file = "def_key \"k\"
  scroll_up";
  let file_remaining = "  scroll_up";

  let def_key_res = def_key(cstr(file));
  assert_eq!(def_key_res, Ok((cstr(file_remaining), cstr("k"))));
}

#[test]
fn parse_action() {
  let file = "  scroll_up
  scroll_down";
  let file_remaining = "  scroll_down";

  let action_res = action(cstr(file));
  assert_eq!(action_res, Ok((cstr(file_remaining), cstr("scroll_up"))));
}

#[test]
fn parse_key_action() {
  let file = "def_key \"k\"
  scroll_up
def_key \"j\"
  scroll_down";
  let file_remaining = "def_key \"j\"
  scroll_down";

  let action_res = key_actions(cstr(file));
  assert_eq!(action_res, Ok((cstr(file_remaining), (cstr("k"), vec![cstr("scroll_up")]))));
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

  let actions_res = key_actions(cstr(file));
  assert_eq!(
    actions_res,
    Ok((cstr(file_remaining), (cstr("k"), vec![cstr("scroll_up"), cstr("scroll_down")])))
  );
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

  let actions_res = key_actions_aggregator(cstr(file));
  assert_eq!(
    actions_res,
    Ok((
      cstr(file_remaining),
      vec![
        (cstr("k"), vec![cstr("scroll_up"), cstr("scroll_down")]),
        (cstr("j"), vec![cstr("scroll_down")])
      ]
    ))
  );
}

#[test]
fn parse_comment() {
  let files = ["# This is a comment", "### This is a title ###"];
  let file_remaining = "";

  for f in files.iter() {
    let comment_res = comment(cstr(f));
    assert_eq!(comment_res, Ok((cstr(file_remaining), cstr(""))));
  }
}

#[test]
fn parse_ignored_line() {
  let files = [
    "# This is a comment
",
    "### This is a title ###
",
  ];
  let file_remaining = "";

  for f in files.iter() {
    let comment_res = ignored_line(cstr(f));
    assert_eq!(comment_res, Ok((cstr(file_remaining), cstr(""))));
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

  let actions_res = key_actions_aggregator(cstr(file));
  assert_eq!(
    actions_res,
    Ok((
      cstr(file_remaining),
      vec![
        (cstr("k"), vec![cstr("scroll_up"), cstr("scroll_down")]),
        (cstr("j"), vec![cstr("scroll_down")])
      ]
    ))
  );
}

#[test]
fn parse_column() {
  let file = "(20)[yellow]{a}";
  let file_remaining = "";

  let column_res = column(cstr(file));
  assert_eq!(column_res, Ok((cstr(file_remaining), (20, false, cstr("yellow"), cstr("a")))));
}

#[test]
fn parse_column_special_content() {
  let file = "(10)[blue]{t|f:Title}";
  let file_remaining = "";

  let column_res = column(cstr(file));
  assert_eq!(column_res, Ok((cstr(file_remaining), (10, false, cstr("blue"), cstr("t|f:Title")))));
}

#[test]
fn parse_column_nocolor() {
  let file = "(10)[]{a}";
  let file_remaining = "";

  let column_res = column(cstr(file));
  assert_eq!(column_res, Ok((cstr(file_remaining), (10, false, cstr(""), cstr("a")))));
}

#[test]
fn parse_fixed_column() {
  let file = "(5f)[red]{b}";
  let file_remaining = "";

  let column_res = column(cstr(file));
  assert_eq!(column_res, Ok((cstr(file_remaining), (5, true, cstr("red"), cstr("b")))));
}

#[test]
fn parse_columns() {
  let file = "(20)[]{a} (6f)[green]{NE} (50)[white]{t|f:Title} (20)[cyan]{b} (7f)[magenta]{l}";
  let file_remaining = "";

  let columns_res = get_columns_format(cstr(file));
  assert_eq!(
    columns_res,
    Ok((
      cstr(file_remaining),
      vec![
        (20, false, cstr(""), cstr("a")),
        (6, true, cstr("green"), cstr("NE")),
        (50, false, cstr("white"), cstr("t|f:Title")),
        (20, false, cstr("cyan"), cstr("b")),
        (7, true, cstr("magenta"), cstr("l"))
      ]
    ))
  );
}
