// src/coreui.rs

use crate::{
  GetRect, Rect,
};
use crossterm::style::Color;


pub trait Draw {
  fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()>;
}

// corners 
// square
pub const A_SQR: char = '\u{250C}';
pub const B_SQR: char = '\u{2510}';
pub const C_SQR: char = '\u{2514}';
pub const D_SQR: char = '\u{2518}';
// round
pub const A_RND: char = '\u{256D}';
pub const B_RND: char = '\u{256E}';
pub const C_RND: char = '\u{2570}';
pub const D_RND: char = '\u{256F}';
// lines
pub const X_LINE: char = '\u{2500}';
pub const Y_LINE: char = '\u{2502}';
// brackets
// tortoise shell square bracket (hot)
pub const OPEN_TORT:  char = '\u{2997}';
pub const CLOSE_TORT: char = '\u{2998}';
// super square bracket (hot)
pub const OPEN_SQR:  char = '\u{27E6}';
pub const CLOSE_SQR: char = '\u{27E7}';
// brack with quill (pretty good)
pub const OPEN_E:  char = '\u{2045}';
pub const CLOSE_E: char = '\u{2046}';
// integrals (not bad)
pub const OPEN_INT:  char = '\u{2320}';
pub const CLOSE_INT: char = '\u{2321}';
// ceiling / floor (not bad)
pub const OPEN_L:  char = '\u{2308}';
pub const CLOSE_L: char = '\u{230B}';

pub fn parse_color(v: &toml::Value) -> Result<Color, String> {
  match v {
    toml::Value::String(s) => 
      if let Some('#') = s.chars().next() {
        parse_hex_color(&s[1..])
      } else {
        parse_color_name(&s)
      }
    _ => Err(format!("could not parse color from value {v}")),
  }
}

pub fn parse_color_name(s: &str) -> Result<Color, String> {
  match s {
    "Red"         | "red"         => Ok(Color::Red),
    "Yellow"      | "yellow"      => Ok(Color::Yellow),
    "Green"       | "green"       => Ok(Color::Green),
    "Cyan"        | "cyan"        => Ok(Color::Cyan),
    "Blue"        | "blue"        => Ok(Color::Blue),
    "Magenta"     | "magenta"     => Ok(Color::Magenta),
    "Black"       | "black"       => Ok(Color::Black),
    "White"       | "white"       => Ok(Color::White),
    "Grey"        | "grey"     | 
    "Gray"        | "gray"        => Ok(Color::Grey),
    "DarkGrey"    | "darkgrey" | 
    "DarkGray"    | "darkgray"    => Ok(Color::DarkGrey),
    "DarkRed"     | "darkred"     => Ok(Color::DarkRed),
    "DarkYellow"  | "darkyellow"  => Ok(Color::DarkYellow),
    "DarkGreen"   | "darkgreen"   => Ok(Color::DarkGreen),
    "DarkCyan"    | "darkcyan"    => Ok(Color::DarkCyan),
    "DarkBlue"    | "darkblue"    => Ok(Color::DarkBlue),
    "DarkMagenta" | "darkmagenta" => Ok(Color::DarkMagenta),
    _ => Err(format!("could not parse color from value {s}"))
  }
}

pub fn parse_hex_color(s: &str) -> Result<Color, String> {
  fn try_hex(c: char) -> Result<u8, String> {
    match c {
      '0' => Ok(0),  '1' => Ok(1),  '2' => Ok(2),  '3' => Ok(3),
      '4' => Ok(4),  '5' => Ok(5),  '6' => Ok(6),  '7' => Ok(7),
      '8' => Ok(8),  '9' => Ok(9),  'a' => Ok(10), 'b' => Ok(11),
      'c' => Ok(12), 'd' => Ok(13), 'e' => Ok(14), 'f' => Ok(15),
      _   => Err(format!("{c} is not a hex character")),
    }
  }
  let mut c = s.chars();
  let r1 = c
    .next()
    .ok_or("missing first red".into())
    .and_then(|c| try_hex(c))?;
  let r2 = c
    .next()
    .ok_or("missing second red".into())
    .and_then(|c| try_hex(c))?;
  let g1 = c
    .next()
    .ok_or("missing first green".into())
    .and_then(|c| try_hex(c))?;
  let g2 = c
    .next()
    .ok_or("missing second green".into())
    .and_then(|c| try_hex(c))?;
  let b1 = c
    .next()
    .ok_or("missing first blue".into())
    .and_then(|c| try_hex(c))?;
  let b2 = c
    .next()
    .ok_or("missing second blue".into())
    .and_then(|c| try_hex(c))?;
  let r = 16 * r1 + r2;
  let g = 16 * g1 + g2;
  let b = 16 * b1 + b2;
  Ok(Color::Rgb {r, g, b})
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Style {
  pub underline: bool,
  pub bold:      bool,
  pub fg:        Option<Color>,
  pub bg:        Option<Color>,
}

impl From<TextStyle> for Style {
  fn from(item: TextStyle) -> Self {item.style}
}

impl crossterm::Command for Style {
  fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    use crossterm::{
      style::{SetStyle, ContentStyle, Attribute, Attributes},
    };
    let mut contentstyle = ContentStyle::new();
    contentstyle.foreground_color = self.fg;
    contentstyle.background_color = self.bg;
    let mut attributes = Attributes::none();
    if self.bold {
      attributes.set(Attribute::Bold);
    }
    if self.underline {
      attributes.set(Attribute::Underlined);
    }
    contentstyle.attributes = attributes;
    SetStyle(contentstyle).write_ansi(f)?;
    Ok(())
  }
}

#[derive(Copy, Debug, Clone)]
pub struct Margins {
  pub north: u16,
  pub south: u16,
  pub east:  u16,
  pub west:  u16,
}

impl Default for Margins {
  fn default() -> Self {
    Self {
      north: 1, 
      south: 1, 
      east:  1, 
      west:  1,
    }
  }
}

impl Margins {
  pub fn get_rect<V: GetRect>(&self, view: V) -> Rect {
    view
      .get_rect()
      .crop_north(self.north)
      .crop_south(self.south)
      .crop_east(self.east)
      .crop_west(self.west)
  }
}

#[derive(Copy, Debug, Clone)]
pub struct BorderStyle {
  pub style: Style,
  pub x:     char,
  pub y:     char,
  pub a:     char,
  pub b:     char,
  pub c:     char,
  pub d:     char,
  pub open:  char,
  pub close: char,
}

impl Default for BorderStyle {
  fn default() -> Self {
    Self {
      style: Style::default(),
      x:     X_LINE,
      y:     Y_LINE,
      a:     A_SQR,
      b:     B_SQR,
      c:     C_SQR,
      d:     D_SQR,
      open:  ' ',
      close: ' ',
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct TextStyle {
  pub style: Style,
  pub wrap:  bool,
}

impl Default for TextStyle {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}

impl std::ops::Deref for TextStyle {
  type Target = Style;
  fn deref(&self) -> &Self::Target {&self.style}
}

impl TextStyle {
  // split at spaces within width and split at lines
  pub fn print(&self, width: usize, text: &str) -> Vec<Vec<char>> {
    if text.len() == 0 {
      vec![vec![]]
    } else if self.wrap {
      text
        .lines()
        .flat_map(|line| crate::util::get_wrapped_text(line, width))
        .collect()
    } else {
      text
        .lines()
        .map(|line| line.chars().collect())
        .collect()
    }
  }
}
