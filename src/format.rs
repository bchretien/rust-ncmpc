extern crate ncurses;
extern crate nom;

use constants::Color;
use ncurses as nc;
use parser::{ParserError, get_columns_format};
use std::fmt;

/// Column type for playlist display.
#[derive(Clone, PartialEq, Debug)]
pub enum SongProperty {
  Album,
  AlbumArtist,
  Artist,
  Comment,
  Composer,
  Date,
  Directory,
  Disc,
  Filename,
  Genre,
  Length,
  Performer,
  Priority,
  Title,
  Track,
  TrackFull,
}

impl fmt::Display for SongProperty {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use format::SongProperty::*;
    write!(
      f,
      "{}",
      match *self {
        Album => "Album",
        AlbumArtist => "Album Artist",
        Artist => "Artist",
        Comment => "Comment",
        Composer => "Composer",
        Date => "Date",
        Directory => "Directory",
        Disc => "Disc",
        Filename => "Filename",
        Genre => "Genre",
        Length => "Time",
        Performer => "Performer",
        Priority => "Priority",
        Title => "Title/Filename",
        Track => "Track",
        TrackFull => "Full Track",
      }
    )
  }
}

/// Column used to display the current playlist.
#[derive(Clone, PartialEq, Debug)]
pub struct Column {
  /// Type of the column.
  pub column_type: SongProperty,
  /// Width of the column (in percent, or number of characters if fixed)
  pub width: i32,
  /// Whether the column width is fixed.
  pub is_fixed: bool,
  /// Color of the column's text.
  pub color: Color,
}

fn get_column_type(s: &str) -> Result<SongProperty, ParserError> {
  let c = s.chars().next().unwrap_or(' ');
  // TODO: use hashmap
  if c == 'l' {
    Ok(SongProperty::Length)
  } else if c == 'f' {
    Ok(SongProperty::Filename)
  } else if c == 'D' {
    Ok(SongProperty::Directory)
  } else if c == 'a' {
    Ok(SongProperty::Artist)
  } else if c == 'A' {
    Ok(SongProperty::AlbumArtist)
  } else if c == 't' {
    Ok(SongProperty::Title)
  } else if c == 'b' {
    Ok(SongProperty::Album)
  } else if c == 'y' {
    Ok(SongProperty::Date)
  } else if c == 'n' {
    Ok(SongProperty::Track)
  } else if c == 'N' {
    Ok(SongProperty::TrackFull)
  } else if c == 'g' {
    Ok(SongProperty::Genre)
  } else if c == 'c' {
    Ok(SongProperty::Composer)
  } else if c == 'p' {
    Ok(SongProperty::Performer)
  } else if c == 'd' {
    Ok(SongProperty::Disc)
  } else if c == 'C' {
    Ok(SongProperty::Comment)
  } else if c == 'P' {
    Ok(SongProperty::Priority)
  } else {
    Err(ParserError::Parse(0))
  }
}

fn get_color(s: &str) -> Color {
  if s == "black" {
    nc::COLOR_BLACK
  } else if s == "red" {
    nc::COLOR_RED
  } else if s == "green" {
    nc::COLOR_GREEN
  } else if s == "yellow" {
    nc::COLOR_YELLOW
  } else if s == "blue" {
    nc::COLOR_BLUE
  } else if s == "magenta" {
    nc::COLOR_MAGENTA
  } else if s == "cyan" {
    nc::COLOR_CYAN
  } else if s == "white" {
    nc::COLOR_WHITE
  } else {
    -1
  }
}

pub fn generate_columns(format: &str) -> Result<Vec<Column>, ParserError> {
  let res = get_columns_format(format);
  match res {
    nom::IResult::Done(_i, o) => {
      let mut columns = Vec::<Column>::default();
      for c in o {
        columns.push(Column {
          column_type: try!(get_column_type(c.3)),
          width: c.0,
          is_fixed: c.1,
          color: get_color(c.2),
        });
      }
      return Ok(columns);
    }
    _ => Err(ParserError::Parse(0)),
  }
}

/// Format flags used by ncurses.
#[derive(Clone, PartialEq, Debug)]
pub enum Format {
  None,
  Bold,
  NoBold,
  Underline,
  NoUnderline,
  Reverse,
  NoReverse,
  AltCharset,
  NoAltCharset,
}

pub fn get_format(s: &str) -> Result<Format, ParserError> {
  if s.is_empty() {
    return Ok(Format::None);
  }

  let mut iter = s.chars();
  let mut c = iter.next().unwrap_or(' ');
  // TODO: use hashmap
  if c == '/' {
    c = iter.next().unwrap_or(' ');
    if c == 'b' {
      return Ok(Format::NoBold);
    } else if c == 'u' {
      return Ok(Format::NoUnderline);
    } else if c == 'r' {
      return Ok(Format::NoReverse);
    } else if c == 'a' {
      return Ok(Format::NoAltCharset);
    }
  } else if c == 'b' {
    return Ok(Format::Bold);
  } else if c == 'u' {
    return Ok(Format::Underline);
  } else if c == 'r' {
    return Ok(Format::Reverse);
  } else if c == 'a' {
    return Ok(Format::AltCharset);
  }
  Err(ParserError::Parse(0))
}

/// Expression used for song formats.
#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
  String(String),
  Color(Color),
  Format(Format),
  SongProperty(SongProperty),
}

#[test]
fn check_get_format() {
  assert_eq!(get_format("").unwrap(), Format::None);
  assert_eq!(get_format("b").unwrap(), Format::Bold);
  assert_eq!(get_format("/b").unwrap(), Format::NoBold);
  assert_eq!(get_format("u").unwrap(), Format::Underline);
  assert_eq!(get_format("/u").unwrap(), Format::NoUnderline);
  assert_eq!(get_format("r").unwrap(), Format::Reverse);
  assert_eq!(get_format("/r").unwrap(), Format::NoReverse);
  assert_eq!(get_format("a").unwrap(), Format::AltCharset);
  assert_eq!(get_format("/a").unwrap(), Format::NoAltCharset);
  assert!(get_format("c").is_err());
  assert!(get_format("/d").is_err());
}

#[test]
fn check_expression() {
  // TODO: improve test
  let mut v = Vec::<Expression>::default();
  v.push(Expression::String("test".to_string()));
  v.push(Expression::Color(nc::COLOR_RED));
  v.push(Expression::Format(Format::NoBold));
  v.push(Expression::SongProperty(SongProperty::Album));
}
