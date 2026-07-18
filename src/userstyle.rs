// src/userstyle.rs

use crate::{
  Assign, 
  UserTable,
  MarginParams,
  BorderParams,
  TextParams,
  TabText,
  GemTag,
  GemText,
  FrameParams,
  Style,
  color,
  constants::*,
};
use toml::Value;


#[derive(Copy, Clone, Default, Debug)]
pub struct SystemStyleParams {
  pub text_margin:     MarginParams,
  pub screen_margin:   MarginParams,
  pub border:          Option<BorderParams>,
  pub general:         TextParams,
  pub banner:          TextParams,
  pub info:            TextParams,
  pub text:            TextParams,
  pub heading3:        TextParams,
  pub heading2:        TextParams,
  pub heading1:        TextParams,
  pub preformat:       TextParams,
  pub link:            TextParams,
  pub error:           TextParams,
  pub quote:           TextParams,
  pub list:            TextParams,
} 


impl SystemStyleParams {
  pub fn get_frame_params(&self) -> FrameParams {
    FrameParams::init()
      .screen_margin(self.screen_margin)
      .text_margin(self.text_margin)
      .banner_style(&self.banner)
      .footer_style(&self.banner)
      .margin_style(&self.general)
      .border_style(self.border)
  }


  pub fn get_dialog_frame_params(&self) -> FrameParams {
    FrameParams::init()
      .banner_style(&self.banner)
      .footer_style(&self.banner)
      .margin_style(&self.general)
      .border_style(self.border)
  }


  pub fn get_tab_text_params(&self, text: &TabText) -> TextParams {
    match text {
      TabText::Gemini(gemtext) => self.get_gem_text_params(gemtext),
      _ => TextParams::default(),
    }
  }


  pub fn get_gem_text_params(&self, text: &GemText) -> TextParams {
    self.get_gem_tag_params(&text.tag)
  }


  pub fn get_gem_tag_params(&self, tag: &GemTag) -> TextParams {
    match tag {
      GemTag::HeadingOne   => self.heading1.into(),
      GemTag::HeadingTwo   => self.heading2.into(),
      GemTag::HeadingThree => self.heading3.into(),
      GemTag::Text         => self.text.into(),
      GemTag::PreFormat    => self.preformat.into(),
      GemTag::Link(_)      => self.link.into(),
      GemTag::ListItem     => self.list.into(),
      GemTag::Quote        => self.quote.into(),
    }
  }
}


impl Assign for SystemStyleParams {
  type Field = StyleTableField;

  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    match (f, v) {
      (StyleTableField::Border, Value::Table(v)) => {
        self.border = Some(BorderParams::default().read_table(v)?);
      }
      (StyleTableField::Text(f), Value::Table(v)) => {
        let v = TextParams::default().read_table(v)?;
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
        let v = MarginParams::default().read_table(v)?;
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


impl Assign for Style {
  type Field = StyleField;

  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    match (f, v) {
      (StyleField::Color(f), v) => {
        let v = color::parse_color(&v).map_err(|e| format!("{v:?} : {e}"))?;
        match f {
          ColorField::Fg => self.fg = Some(v),
          ColorField::Bg => self.bg = Some(v),
        }
      }
      (StyleField::Attribute(f), Value::Boolean(v)) => {
        match f {
          AttributeField::Bold => self.bold = v,
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


impl Assign for MarginParams {
  type Field = MarginParamsField;

  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    match (f, v) {
      (f, Value::Integer(v)) => {
        let v = u16::try_from(v).map_err(|e| format!("{v:?} : {e}"))?;
        match f {
          MarginParamsField::North => self.north = v,
          MarginParamsField::South => self.south = v,
          MarginParamsField::East => self.east = v,
          MarginParamsField::West => self.west = v,
        }
      }
      (_, v) => return Err(
        format!("margin must be a number, not {v:?}")
      )
    }
    Ok(())
  }
}


impl Assign for BorderParams {
  type Field = BorderParamsField;

  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    match (f, v) {
      (BorderParamsField::Style(f), v) => {
        self.style.assign(f, v)?;
      }
      (BorderParamsField::Corner, Value::String(v)) => {
        match v.as_str() {
          "square" => {
            self.northwest = A_SQR;
            self.northeast = B_SQR;
            self.southwest = C_SQR;
            self.southeast = D_SQR;
          }
          "round" => {
            self.northwest = A_RND;
            self.northeast = B_RND;
            self.southwest = C_RND;
            self.southeast = D_RND;
          }
          s => return Err(
            format!("Corner field does not contain {s}")
          ),
        }
      }
      (BorderParamsField::Bracket, Value::String(v)) => {
        match v.as_str() {
          "space" => {
            self.open = ' ';
            self.close = ' ';
          }
          "tortoise" | "tort" | "t" => {
            self.open = OPEN_TORT;
            self.close = CLOSE_TORT;
          }
          "integral" | "int"  | "i" | "j" | "J" => {
            self.open = OPEN_INT;
            self.close = CLOSE_INT;
          }
          "square" | "sqr" => {
            self.open = OPEN_SQR;
            self.close = CLOSE_SQR;
          }
          "E" | "e" => {
            self.open = OPEN_E;
            self.close = CLOSE_E;
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


impl Assign for TextParams {
  type Field = TextStyleParamsField;

  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    match (f, v) {
      (TextStyleParamsField::Wrap, Value::Boolean(v)) => {
        self.wrap = v;
      }
      (TextStyleParamsField::Style(f), v) => {
        self.style.assign(f, v)?;
      }
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}


#[derive(Debug)]
pub enum ColorField {
  Fg, Bg
}


#[derive(Debug)]
pub enum AttributeField {
  Bold, Underline
}


#[derive(Debug)]
pub enum StyleMarginField {
  Text, Screen
}


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


impl std::str::FromStr for StyleTableField {
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
pub enum MarginParamsField {
  North, South, East, West
}


impl std::str::FromStr for MarginParamsField {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "north" | "n" => Ok(Self::North),
      "south" | "s" => Ok(Self::South),
      "east" | "e" => Ok(Self::East),
      "west" | "w" => Ok(Self::West),
      s => Err(format!("Margin table does not contain field {s}")),
    }
  }
}


#[derive(Debug)]
pub enum StyleField {
  Color(ColorField), 
  Attribute(AttributeField)
}


impl std::str::FromStr for StyleField {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "fg" => Ok(Self::Color(ColorField::Fg)),
      "bg" => Ok(Self::Color(ColorField::Bg)),
      "bold" => Ok(Self::Attribute(AttributeField::Bold)),
      "underline" => Ok(Self::Attribute(AttributeField::Underline)),
      s => Err(format!("Style table does not contain field {s}")),
    }
  }
}


#[derive(Debug)]
pub enum TextStyleParamsField {
  Wrap, 
  Style(StyleField)
}


impl std::str::FromStr for TextStyleParamsField {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "wrap" => Ok(Self::Wrap),
      s => StyleField::from_str(s).map(|s| Self::Style(s))
    }
  }
}

#[derive(Debug)]
pub enum BorderParamsField {
  Style(StyleField), Corner, Bracket
}


impl std::str::FromStr for BorderParamsField {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "corner" => Ok(Self::Corner),
      "bracket" => Ok(Self::Bracket),
      s => StyleField::from_str(s).map(|s| Self::Style(s))
    }
  }
}
