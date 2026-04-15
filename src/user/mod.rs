// src/usr/mod.rs

mod style;
mod keys;

pub use self::style::StyleModTable;
pub use self::keys::KeysTable;

use crate::{
  screen::Rect,
  text::StyledText,
  widget::{Frame, TextBox},
  protocol::{GemText, GemTag, GemDoc},
};
use toml::{Table, Value};
use std::{fs, str::{FromStr}};


pub trait UserTable<F>: Sized 
where F: FromStr<Err = String> 
{
  fn try_assign(&mut self, field: F, value: Value) -> Result<(), String>;

  fn read_table(mut self, table: Table) -> Result<Self, String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.try_assign(field, value)?;
    }
    Ok(self)
  }
}
#[derive(Debug)]
enum UserField {
  InitUrl, Timeout, Style, Keys
}
impl FromStr for UserField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "init_url" => Ok(Self::InitUrl),
      "timeout"  => Ok(Self::Timeout),
      "style"    => Ok(Self::Style),
      "keys"     => Ok(Self::Keys),
      s          => Err(format!("No field {} in User table", s)),
    }
  }
}
#[derive(Clone)]
pub struct User {
  pub override_style: Option<String>,
  pub override_keys:  Option<String>,
  pub timeout:        u64,
  pub init_url:       String,
  pub style:          StyleModTable,
  pub keys:           KeysTable,
} 
impl FromStr for User {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    Self::default().read_table(table)
  }
}
impl Default for User {
  fn default() -> Self {
    Self {
      override_style: None,
      override_keys:  None,
      timeout:        10,
      init_url:       "gemini://geminiprotocol.net/".into(),
      style:          StyleModTable::default(),
      keys:           KeysTable::default(),
    }
  }
}
impl UserTable<UserField> for User {
  fn try_assign(&mut self, field: UserField, value: Value) 
    -> Result<(), String> 
  {
    match (field, value) {
      (UserField::InitUrl, Value::String(v)) => {
        self.init_url = v.into();
      }
      // read style from another file
      (UserField::Style, Value::String(path)) => {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let table = text.parse::<Table>().map_err(|e| e.to_string())?;
        self.style = StyleModTable::default().read_table(table)?;
      }
      // read keys from another file
      (UserField::Keys, Value::String(path)) => {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let table = text.parse::<Table>().map_err(|e| e.to_string())?;
        self.keys = KeysTable::default().read_table(table)?;
      }
      // read style from this file
      (UserField::Style, Value::Table(v)) => {
        self.style = StyleModTable::default().read_table(v)?;
      }
      // read keys from this file
      (UserField::Keys, Value::Table(v)) => {
        self.keys = KeysTable::default().read_table(v)?;
      }
      // read keys from this file
      (UserField::Timeout, Value::Integer(v)) => {
        self.timeout = u64::try_from(v).map_err(|e| e.to_string())?;
      }
      (f, v) => 
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
impl User {
  pub fn get_frame(&self, screen: &Rect) -> Frame {
    Frame::new(
        &screen, 
        self.style.border.clone(),
        self.style.screen_margin.clone(),
        self.style.text_margin.clone()
      ).with_banner_style(&self.style.banner.style)
      .with_margin_style(&self.style.general.style)
  }
  pub fn get_gem_textbox(&self, rect: &Rect, gem: &GemDoc) -> TextBox {
    TextBox::new(
        gem.doc.iter().map(|gem| self.gem_to_styled(gem)).collect(),
        rect,
      ).with_style(&self.style.general.style)
  }
  pub fn get_info_styledtext(&self, msg: &str) -> StyledText {
    StyledText::from(msg)
      .with_style(&self.style.info.style)
      .wrap(self.style.info.wrap)
  }
  pub fn gem_to_styled(&self, gemtext: &GemText) -> StyledText {
    match gemtext.tag {
      GemTag::HeadingOne => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.header1.style)
          .wrap(self.style.header1.wrap),
      GemTag::HeadingTwo => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.header2.style)
          .wrap(self.style.header2.wrap),
      GemTag::HeadingThree => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.header3.style)
          .wrap(self.style.header3.wrap),
      GemTag::Text => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.text.style)
          .wrap(self.style.text.wrap),
      GemTag::PreFormat => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.preformat.style)
          .wrap(self.style.preformat.wrap),
      GemTag::Link(_, _) => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.link.style)
          .wrap(self.style.link.wrap),
      GemTag::BadLink(_) => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.error.style)
          .wrap(self.style.error.wrap),
      GemTag::ListItem => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.list.style)
          .wrap(self.style.list.wrap),
      GemTag::Quote => 
        StyledText::from(gemtext.txt.as_str())
          .with_style(&self.style.quote.style)
          .wrap(self.style.quote.wrap),
    }
  }
}
