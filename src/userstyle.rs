// src/style.rs

use crate::{
  UserTable, 
  Rect, 
  Margins,
  BorderStyle,
  TextStyle,
  StyledText,
  GemTag,
  GemText,
  Frame,
  Style,
  uiprimitives,
};
use toml::Value;
use std::str::FromStr;


#[derive(Clone, Debug)]
pub enum ColorField {Fg, Bg}

#[derive(Clone, Debug)]
pub enum AttributeField {Bold, Underline}

#[derive(Debug)]
pub enum StyleMarginField {Text, Screen}

#[derive(Debug)]
pub enum StyleTextField {
  General,
  Banner,
  Info,
  Text,
  Heading3,
  Heading2,
  Heading1,
  Preformat,
  Link,
  Error,
  Quote,
  List,
}

#[derive(Debug)]
pub enum StyleTableField {
  Border, 
  Margin(StyleMarginField), 
  Text(StyleTextField),
}
impl FromStr for StyleTableField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "border"          => Ok(Self::Border),
      "text_margin"     => Ok(Self::Margin(StyleMarginField::Text)),
      "screen_margin"   => Ok(Self::Margin(StyleMarginField::Screen)),
      "general"         => Ok(Self::Text(StyleTextField::General)),
      "banner"          => Ok(Self::Text(StyleTextField::Banner)),
      "info"            => Ok(Self::Text(StyleTextField::Info)),
      "text"            => Ok(Self::Text(StyleTextField::Text)),
      "heading3" | "h3" => Ok(Self::Text(StyleTextField::Heading3)),
      "heading2" | "h2" => Ok(Self::Text(StyleTextField::Heading2)),
      "heading1" | "h1" => Ok(Self::Text(StyleTextField::Heading1)),
      "preformat"       => Ok(Self::Text(StyleTextField::Preformat)),
      "link"            => Ok(Self::Text(StyleTextField::Link)),
      "error"           => Ok(Self::Text(StyleTextField::Error)),
      "quote"           => Ok(Self::Text(StyleTextField::Quote)),
      "list"            => Ok(Self::Text(StyleTextField::List)),
      s => Err(format!("Style table does not contain field {s}")),
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
      s => Err(format!("Margin table does not contain field {s}")),
    }
  }
}

#[derive(Clone, Debug)]
pub enum StyleField {
  Color(ColorField), 
  Attribute(AttributeField),
}
impl FromStr for StyleField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "fg"        => Ok(Self::Color(ColorField::Fg)),
      "bg"        => Ok(Self::Color(ColorField::Bg)),
      "bold"      => Ok(Self::Attribute(AttributeField::Bold)),
      "underline" => Ok(Self::Attribute(AttributeField::Underline)),
      s => Err(format!("Style table does not contain field {s}")),
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

impl UserTable<StyleField> for Style {
  fn assign(&mut self, field: StyleField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (StyleField::Color(f), v) => {
        let v = uiprimitives::parse_color(&v)
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
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}

impl UserTable<MarginField> for Margins {
  fn assign(&mut self, field: MarginField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (f, Value::Integer(v)) => {
        let v = u16::try_from(v).map_err(|e| format!("{v:?} : {e}"))?;
        match f {
          MarginField::North => self.north = v,
          MarginField::South => self.south = v,
          MarginField::East  => self.east  = v,
          MarginField::West  => self.west  = v,
        }
      }
      (f, v) => return Err(
        format!("margin must be a number, not {v:?}")
      )
    }
    Ok(())
  }
}

impl UserTable<BorderField> for BorderStyle {
  fn assign(&mut self, field: BorderField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (BorderField::Style(f), v) => {
        self.style.assign(f, v)?;
      }
      (BorderField::Corner, Value::String(v)) => {
        match v.as_str() {
          "square" => {
            self.a = uiprimitives::A_SQR;
            self.b = uiprimitives::B_SQR;
            self.c = uiprimitives::C_SQR;
            self.d = uiprimitives::D_SQR;
          }
          "round" => {
            self.a = uiprimitives::A_RND;
            self.b = uiprimitives::B_RND;
            self.c = uiprimitives::C_RND;
            self.d = uiprimitives::D_RND;
          }
          s => return Err(
            format!("Corner field does not contain {s}")
          ),
        }
      }
      (BorderField::Bracket, Value::String(v)) => {
        match v.as_str() {
          "space" => {
            self.open  = ' ';
            self.close = ' ';
          }
          "tortoise" | "tort" | "t" => {
            self.open  = uiprimitives::OPEN_TORT;
            self.close = uiprimitives::CLOSE_TORT;
          }
          "integral" | "int"  | "i" | "j" | "J" => {
            self.open  = uiprimitives::OPEN_INT;
            self.close = uiprimitives::CLOSE_INT;
          }
          "square" | "sqr" => {
            self.open  = uiprimitives::OPEN_SQR;
            self.close = uiprimitives::CLOSE_SQR;
          }
          "E" | "e" => {
            self.open  = uiprimitives::OPEN_E;
            self.close = uiprimitives::CLOSE_E;
          }
          s => return Err(
            format!("Bracket field does not contain {s}")
          ),
        }
      }
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}

impl UserTable<TextField> for TextStyle {
  fn assign(&mut self, field: TextField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (TextField::Wrap, Value::Boolean(v)) => {
        self.wrap = v;
      }
      (TextField::Style(f), v) => {
        self.style.assign(f, v)?;
      }
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}

#[derive(Clone, Default, Debug)]
pub struct UserStyle {
  pub text_margin:     Margins,
  pub screen_margin:   Margins,
  pub border:          BorderStyle,
  pub general:         TextStyle,
  pub banner:          TextStyle,
  pub info:            TextStyle,
  pub text:            TextStyle,
  pub heading3:        TextStyle,
  pub heading2:        TextStyle,
  pub heading1:        TextStyle,
  pub preformat:       TextStyle,
  pub link:            TextStyle,
  pub error:           TextStyle,
  pub quote:           TextStyle,
  pub list:            TextStyle,
} 
impl UserTable<StyleTableField> for UserStyle {
  fn assign(&mut self, field: StyleTableField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (StyleTableField::Border, Value::Table(v)) => {
        self.border = BorderStyle::default().read_table(v)?;
      }
      (StyleTableField::Text(f), Value::Table(v)) => {
        let v = TextStyle::default().read_table(v)?;
        match f {
          StyleTextField::General   => self.general   = v,
          StyleTextField::Banner    => self.banner    = v,
          StyleTextField::Info      => self.info      = v,
          StyleTextField::Text      => self.text      = v,
          StyleTextField::Heading3  => self.heading3  = v,
          StyleTextField::Heading2  => self.heading2  = v,
          StyleTextField::Heading1  => self.heading1  = v,
          StyleTextField::Preformat => self.preformat = v,
          StyleTextField::Link      => self.link      = v,
          StyleTextField::Error     => self.error     = v,
          StyleTextField::Quote     => self.quote     = v,
          StyleTextField::List      => self.list      = v,
        }
      }
      (StyleTableField::Margin(f), Value::Table(v)) => {
        let v = Margins::default().read_table(v)?;
        match f {
          StyleMarginField::Text   => self.text_margin   = v,
          StyleMarginField::Screen => self.screen_margin = v,
        }
      }
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}
impl UserStyle {
  pub fn get_frame(&self, screen: Rect) -> Frame {
    Frame::from(screen)
      .screen_margin(self.screen_margin)
      .text_margin(self.text_margin)
      .banner_style(self.banner)
      .footer_style(self.banner)
      .margin_style(self.general)
      .border_style(self.border)
  }

  pub fn get_styled_gemtext(&self, gemtext: &GemText) -> StyledText {
    let mut text: StyledText = match gemtext.tag {
      GemTag::HeadingOne   => self.heading1.into(),
      GemTag::HeadingTwo   => self.heading2.into(),
      GemTag::HeadingThree => self.heading3.into(),
      GemTag::Text         => self.text.into(),
      GemTag::PreFormat    => self.preformat.into(),
      GemTag::Link(_)      => self.link.into(),
      GemTag::ListItem     => self.list.into(),
      GemTag::Quote        => self.quote.into(),
    };
    text.text(&gemtext.to_string())
  }
}
