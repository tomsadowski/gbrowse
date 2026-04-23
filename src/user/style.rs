// src/user/style.rs

use crate::{
  common as c,
  user::UserTable,
  widget::{BorderSpec, MarginSpec, Style},
};
use crossterm::{
  style::{Color},
};
use toml::{Value};
use std::str::FromStr;

pub fn parse_color(v: &Value) -> Result<Color, String> {
  if let Value::String(s) = v {
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
  } else {
    Err(format!("could not parse color from value {}", v))
  }
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
  Border,
  Covered,
  Margin(StyleMarginField),
  Text(StyleTextField),
}
impl FromStr for StyleModField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "border"        => Ok(Self::Border),
      "covered"       => Ok(Self::Covered),
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
#[derive(Clone, Default, Debug)]
pub struct StyleModTable {
  pub text_margin:     MarginSpec,
  pub screen_margin:   MarginSpec,
  pub border:          BorderSpec,
  pub covered:         Style,
  pub general:         TextTable,
  pub banner:          TextTable,
  pub info:            TextTable,
  pub text:            TextTable,
  pub header3:         TextTable,
  pub header2:         TextTable,
  pub header1:         TextTable,
  pub preformat:       TextTable,
  pub link:            TextTable,
  pub error:           TextTable,
  pub quote:           TextTable,
  pub list:            TextTable,
} 
impl UserTable<StyleModField> for StyleModTable {
  fn try_assign(&mut self, field: StyleModField, value: Value) -> Result<(), String> {
    match (field, value) {
      (StyleModField::Border, Value::Table(v)) => {
        self.border = BorderSpec::default().read_table(v)?;
      }
      (StyleModField::Covered, Value::Table(v)) => {
        self.covered = Style::default().read_table(v)?;
      }
      (StyleModField::Text(f), Value::Table(v)) => {
        let v = TextTable::default().read_table(v)?;
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
impl UserTable<MarginField> for MarginSpec {
  fn try_assign(&mut self, field: MarginField, value: Value) -> Result<(), String> {
    match (field, value) {
      (f, Value::Integer(v)) => {
        let v = u16::try_from(v).map_err(|e| format!("{:?} : {}", v, e))?;
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
#[derive(Clone, Debug)]
pub enum ColorField {
  Fg, Bg
}
#[derive(Clone, Debug)]
pub enum AttributeField {
  Bold, Underline
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
impl UserTable<StyleField> for Style {
  fn try_assign(&mut self, field: StyleField, value: Value) -> Result<(), String> {
    match (field, value) {
      (StyleField::Color(f), v) => {
        let v = parse_color(&v).map_err(|e| format!("{:?} : {}", v, e))?;
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
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
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
#[derive(Clone, Debug)]
pub struct TextTable {
  pub style: Style,
  pub wrap:  bool,
}
impl Default for TextTable {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}
impl UserTable<TextField> for TextTable {
  fn try_assign(&mut self, field: TextField, value: Value) -> Result<(), String> {
    match (field, value) {
      (TextField::Wrap, Value::Boolean(v)) => self.wrap = v,
      (TextField::Style(f), v)             => self.style.try_assign(f, v)?,
      (f, v) => 
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
#[derive(Debug, Clone)]
pub enum CornerValue {
  Square, Round
}
impl FromStr for CornerValue {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "square" => Ok(CornerValue::Square),
      "round"  => Ok(CornerValue::Round),
      s => Err(format!("Corner field does not contain {}", s)),
    }
  }
}
#[derive(Debug, Clone)]
pub enum BracketValue {
  Tortoise, E, Integral, Square,
}
impl FromStr for BracketValue {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "tortoise" | "t" | "tort" => Ok(BracketValue::Tortoise),
      "J" | "j" | "int" | 
      "i" | "integral"          => Ok(BracketValue::Integral),
      "square" | "sqr"          => Ok(BracketValue::Square),
      "E" | "e"                 => Ok(BracketValue::E),
      s => Err(format!("Bracket field does not contain {}", s)),
    }
  }
}
#[derive(Debug, Clone)]
pub enum BorderField {
  Style(StyleField),
  Corner,
  Bracket,
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
impl UserTable<BorderField> for BorderSpec {
  fn try_assign(&mut self, field: BorderField, value: Value) -> Result<(), String> {
    match (field, value) {
      (BorderField::Style(f), v) => self.style.try_assign(f, v)?,
      (BorderField::Corner, Value::String(v)) => {
        match CornerValue::from_str(&v)? {
          CornerValue::Square => {
            self.a = c::A_SQR;
            self.b = c::B_SQR;
            self.c = c::C_SQR;
            self.d = c::D_SQR;
          }
          CornerValue::Round => {
            self.a = c::A_RND;
            self.b = c::B_RND;
            self.c = c::C_RND;
            self.d = c::D_RND;
          }
        }
      }
      (BorderField::Bracket, Value::String(v)) => {
        match BracketValue::from_str(&v)? {
          BracketValue::Tortoise => {
            self.open  = c::OPEN_TORT;
            self.close = c::CLOSE_TORT;
          }
          BracketValue::E => {
            self.open  = c::OPEN_E;
            self.close = c::CLOSE_E;
          }
          BracketValue::Square => {
            self.open  = c::OPEN_SQR;
            self.close = c::CLOSE_SQR;
          }
          BracketValue::Integral => {
            self.open  = c::OPEN_INT;
            self.close = c::CLOSE_INT;
          }
        }
      }
      (f, v) => 
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
