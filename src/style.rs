// src/style.rs

use crate::{
  user::UserTable,
  view::Rect,
};
use crossterm::{
  Command,
  style::{
    SetStyle, ContentStyle, Attribute, Attributes, Color
  },
};
use toml::Value;
use std::{
  fmt,
  str::FromStr,
  ops::Deref,
};

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

pub fn parse_color(v: &Value) -> Result<Color, String> {
  match v {
    Value::String(s) => {
      if let Some('#') = s.chars().next() {
        parse_hex_color(&s[1..])
      } else {
        parse_color_name(&s)
      }
    }
    _ => Err(format!("could not parse color from value {}", v)),
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
    _ => Err(format!("could not parse color from value {}", s))
  }
}

pub fn parse_hex_color(s: &str) -> Result<Color, String> {
  fn try_hex(c: char) -> Result<u8, String> {
    match c {
      '0' => Ok(0),  '1' => Ok(1),  '2' => Ok(2),  '3' => Ok(3),
      '4' => Ok(4),  '5' => Ok(5),  '6' => Ok(6),  '7' => Ok(7),
      '8' => Ok(8),  '9' => Ok(9),  'a' => Ok(10), 'b' => Ok(11),
      'c' => Ok(12), 'd' => Ok(13), 'e' => Ok(14), 'f' => Ok(15),
      _   => Err(format!("{} is not a hex character", c)),
    }
  }
  let mut c = s.chars();
  let r1 = c.next()
    .ok_or("missing first red".into())
    .and_then(|c| try_hex(c))?;
  let r2 = c.next()
    .ok_or("missing second red".into())
    .and_then(|c| try_hex(c))?;
  let g1 = c.next()
    .ok_or("missing first green".into())
    .and_then(|c| try_hex(c))?;
  let g2 = c.next()
    .ok_or("missing second green".into())
    .and_then(|c| try_hex(c))?;
  let b1 = c.next()
    .ok_or("missing first blue".into())
    .and_then(|c| try_hex(c))?;
  let b2 = c.next()
    .ok_or("missing second blue".into())
    .and_then(|c| try_hex(c))?;
  let r = 16 * r1 + r2;
  let g = 16 * g1 + g2;
  let b = 16 * b1 + b2;
  Ok(Color::Rgb {r, g, b})
}

#[derive(Clone, Debug)]
pub enum ColorField {
  Fg, Bg
}
#[derive(Clone, Debug)]
pub enum AttributeField {
  Bold, Underline
}
#[derive(Debug)]
pub enum StyleTextField {
  General,
  Banner,
  Info,
  Text,
  Header3,
  Header2,
  Header1,
  Preformat,
  Link,
  Error,
  Quote,
  List,
}
#[derive(Debug)]
pub enum StyleMarginField {
  Text, Screen,
}
#[derive(Debug)]
pub enum StyleModField {
  Border, Margin(StyleMarginField), Text(StyleTextField),
}
impl FromStr for StyleModField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "border"        => Ok(Self::Border),
      "text_margin"   => Ok(Self::Margin(StyleMarginField::Text)),
      "screen_margin" => Ok(Self::Margin(StyleMarginField::Screen)),
      "general"       => Ok(Self::Text(StyleTextField::General)),
      "banner"        => Ok(Self::Text(StyleTextField::Banner)),
      "info"          => Ok(Self::Text(StyleTextField::Info)),
      "text"          => Ok(Self::Text(StyleTextField::Text)),
      "header3"       => Ok(Self::Text(StyleTextField::Header3)),
      "header2"       => Ok(Self::Text(StyleTextField::Header2)),
      "header1"       => Ok(Self::Text(StyleTextField::Header1)),
      "preformat"     => Ok(Self::Text(StyleTextField::Preformat)),
      "link"          => Ok(Self::Text(StyleTextField::Link)),
      "error"         => Ok(Self::Text(StyleTextField::Error)),
      "quote"         => Ok(Self::Text(StyleTextField::Quote)),
      "list"          => Ok(Self::Text(StyleTextField::List)),
      s => Err(format!("Style table does not contain field {}", s)),
    }
  }
}
#[derive(Debug)]
enum MarginField {
  North, South, East, West,
}
impl FromStr for MarginField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "north" | "n" => Ok(Self::North),
      "south" | "s" => Ok(Self::South),
      "east"  | "e" => Ok(Self::East),
      "west"  | "w" => Ok(Self::West),
      s => Err(format!("Margin table does not contain field {}", s)),
    }
  }
}
#[derive(Clone, Debug)]
pub enum StyleField {
  Color(ColorField), Attribute(AttributeField)
}
impl FromStr for StyleField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "fg"        => Ok(Self::Color(ColorField::Fg)),
      "bg"        => Ok(Self::Color(ColorField::Bg)),
      "bold"      => Ok(Self::Attribute(AttributeField::Bold)),
      "underline" => Ok(Self::Attribute(AttributeField::Underline)),
      s => Err(format!("Style table does not contain field {}", s)),
    }
  }
}
#[derive(Clone, Debug)]
enum TextField {
  Wrap, Style(StyleField)
}
impl FromStr for TextField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "wrap" => Ok(Self::Wrap),
      s      => StyleField::from_str(s).map(|s| Self::Style(s))
    }
  }
}
#[derive(Debug, Clone)]
pub enum BorderField {
  Style(StyleField), Corner, Bracket,
}
impl FromStr for BorderField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "corner"  => Ok(Self::Corner),
      "bracket" => Ok(Self::Bracket),
      s         => StyleField::from_str(s).map(|s| Self::Style(s))
    }
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Style {
  pub underline: bool,
  pub bold:      bool,
  pub fg:        Option<Color>,
  pub bg:        Option<Color>,
}
impl Command for Style {
  fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
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
impl UserTable<StyleField> for Style {
  fn try_assign(&mut self, field: StyleField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (StyleField::Color(f), v) => {
        let v = parse_color(&v)
          .map_err(|e| format!("{:?} : {}", v, e))?;
        match f {
          ColorField::Fg => self.fg = Some(v),
          ColorField::Bg => self.bg = Some(v),
        }
      }
      (StyleField::Attribute(f), Value::Boolean(v)) => {
        match f {
          AttributeField::Bold      => self.bold      = v,
          AttributeField::Underline => self.underline = v,
        }
      }
      (f, v) => 
        return Err(
          format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MarginSpec {
  pub north: u16,
  pub south: u16,
  pub east:  u16,
  pub west:  u16,
}
impl Default for MarginSpec {
  fn default() -> Self {
    Self {north: 0, south: 0, east: 0, west: 0}
  }
}
impl UserTable<MarginField> for MarginSpec {
  fn try_assign(&mut self, field: MarginField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (f, Value::Integer(v)) => {
        let v = u16::try_from(v)
          .map_err(|e| format!("{:?} : {}", v, e))?;
        match f {
          MarginField::North => self.north = v,
          MarginField::South => self.south = v,
          MarginField::East  => self.east  = v,
          MarginField::West  => self.west  = v,
        }
      }
      (f, v) => 
        return Err(format!("margin must be a number, not {:?}", v))
    }
    Ok(())
  }
}
impl MarginSpec {
  pub fn get_rect(&self, screen: Rect) -> Rect {
    screen.clone()
      .crop_north(self.north)
      .crop_south(self.south)
      .crop_east(self.east)
      .crop_west(self.west)
  }
}

#[derive(Debug, Clone)]
pub struct BorderSpec {
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
impl Default for BorderSpec {
  fn default() -> Self {
    Self {
      style: Style::default(),
      x:     X_LINE,
      y:     Y_LINE,
      a:     A_SQR,
      b:     B_SQR,
      c:     C_SQR,
      d:     D_SQR,
      open:  OPEN_SQR,
      close: CLOSE_SQR,
    }
  }
}
impl UserTable<BorderField> for BorderSpec {
  fn try_assign(&mut self, field: BorderField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (BorderField::Style(f), v) => {
        self.style.try_assign(f, v)?;
      }
      (BorderField::Corner, Value::String(v)) => {
        match v.as_str() {
          "square" => {
            self.a = A_SQR;
            self.b = B_SQR;
            self.c = C_SQR;
            self.d = D_SQR;
          }
          "round" => {
            self.a = A_RND;
            self.b = B_RND;
            self.c = C_RND;
            self.d = D_RND;
          }
          s => 
            return Err(format!("Corner field does not contain {}", s)),
        }
      }
      (BorderField::Bracket, Value::String(v)) => {
        match v.as_str() {
          "space" => {
            self.open  = ' ';
            self.close = ' ';
          }
          "tortoise" | "tort" | "t" => {
            self.open  = OPEN_TORT;
            self.close = CLOSE_TORT;
          }
          "integral" | 
          "int"  | 
          "i" | 
          "j" | 
          "J" => {
            self.open  = OPEN_INT;
            self.close = CLOSE_INT;
          }
          "square" | "sqr" => {
            self.open  = OPEN_SQR;
            self.close = CLOSE_SQR;
          }
          "E" | "e" => {
            self.open  = OPEN_E;
            self.close = CLOSE_E;
          }
          s => 
            return Err(
              format!("Bracket field does not contain {}", s)),
        }
      }
      (f, v) => 
        return Err(
          format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}

#[derive(Copy, Clone, Debug)]
pub struct TextStyle {
  pub style: Style,
  pub wrap:  bool,
}
impl Deref for TextStyle {
  type Target = Style;
  fn deref(&self) -> &Self::Target {
    &self.style
  }
}
impl Default for TextStyle {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}
impl UserTable<TextField> for TextStyle {
  fn try_assign(&mut self, field: TextField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (TextField::Wrap, Value::Boolean(v)) => {
        self.wrap = v;
      }
      (TextField::Style(f), v) => {
        self.style.try_assign(f, v)?;
      }
      (f, v) => 
        return Err(
          format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}

#[derive(Clone, Default, Debug)]
pub struct StyleModTable {
  pub text_margin:     MarginSpec,
  pub screen_margin:   MarginSpec,
  pub border:          BorderSpec,
  pub general:         TextStyle,
  pub banner:          TextStyle,
  pub info:            TextStyle,
  pub text:            TextStyle,
  pub header3:         TextStyle,
  pub header2:         TextStyle,
  pub header1:         TextStyle,
  pub preformat:       TextStyle,
  pub link:            TextStyle,
  pub error:           TextStyle,
  pub quote:           TextStyle,
  pub list:            TextStyle,
} 
impl UserTable<StyleModField> for StyleModTable {
  fn try_assign(&mut self, field: StyleModField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (StyleModField::Border, Value::Table(v)) => {
        self.border = BorderSpec::default().read_table(v)?;
      }
      (StyleModField::Text(f), Value::Table(v)) => {
        let v = TextStyle::default().read_table(v)?;
        match f {
          StyleTextField::General   => self.general   = v,
          StyleTextField::Banner    => self.banner    = v,
          StyleTextField::Info      => self.info      = v,
          StyleTextField::Text      => self.text      = v,
          StyleTextField::Header3   => self.header3   = v,
          StyleTextField::Header2   => self.header2   = v,
          StyleTextField::Header1   => self.header1   = v,
          StyleTextField::Preformat => self.preformat = v,
          StyleTextField::Link      => self.link      = v,
          StyleTextField::Error     => self.error     = v,
          StyleTextField::Quote     => self.quote     = v,
          StyleTextField::List      => self.list      = v,
        }
      }
      (StyleModField::Margin(f), Value::Table(v)) => {
        let v = MarginSpec::default().read_table(v)?;
        match f {
          StyleMarginField::Text   => self.text_margin   = v,
          StyleMarginField::Screen => self.screen_margin = v,
        }
      }
      (f, v) => 
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
