// src/user.rs

use crate::{
  keys::KeysTable,
  style::StyleModTable,
  tab::Tab,
  widget::{Frame, TextBox},
  view::Rect,
  text::StyledText,
  protocol::{GemText, GemTag, GemDoc},
};
use toml::{Table, Value};
use std::{fs, str::FromStr};


pub const USER_DATA:   &str = "gdata";
pub const USER_INIT:   &str = "init";
pub const USER_URLS:   &str = "urls";
pub const USER_STYLES: &str = "styles";
pub const USER_KEYS:   &str = "keys";
pub const STYLES_PATH: &str = "gdata/styles";
pub const KEYS_PATH:   &str = "gdata/keys";

pub fn get_entries(path: &str) -> Result<Vec<String>, String> {
  let mut vec: Vec<String> = vec![];
  let mut results = fs::read_dir(path).map_err(|e| e.to_string())?;
  for result in results {
    let s = result
      .map_err(|e| e.to_string())?
      .file_name()
      .into_string()
      .map_err(|_| "Could not convert OsString to String".to_string())?;
    vec.push(s);
  }
  Ok(vec)
}

pub fn get_keys_file(name: &str) -> String {
  format!("{}/{}", KEYS_PATH, name)
}

pub fn get_styles_file(name: &str) -> String {
  format!("{}/{}", STYLES_PATH, name)
}

pub trait UserTable<F>: Sized 
where 
  F: FromStr<Err = String> 
{
  fn try_assign(&mut self, field: F, value: Value) -> Result<(), String>;

  fn read_table(mut self, table: Table) -> Result<Self, String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.try_assign(field, value)?;
    }
    Ok(self)
  }

  fn update_from_table(&mut self, table: Table) -> Result<(), String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.try_assign(field, value)?;
    }
    Ok(())
  }

  fn update_from_str(&mut self, s: &str) -> Result<(), String> {  
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    self.update_from_table(table)?;
    Ok(())
  }
}

#[derive(Debug)]
enum UserField {
  InitUrl, 
  SaveFile,
  Timeout, 
  Style, 
  Keys,
}
impl FromStr for UserField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "init_url" => Ok(Self::InitUrl),
      "timeout"  => Ok(Self::Timeout),
      "style"    => Ok(Self::Style),
      "keys"     => Ok(Self::Keys),
      "gsave" | "save_file" 
                 => Ok(Self::SaveFile),
      s          => Err(format!("No field {} in User table", s)),
    }
  }
}
impl ToString for UserField {
  fn to_string(&self) -> String {
    match self {
     Self::InitUrl  => "init url".into(),
     Self::Timeout  => "timeout".into(),
     Self::Style    => "style".into(),
     Self::Keys     => "keys".into(),
     Self::SaveFile => "save file".into(),
    }
  }
}
impl UserField {
  pub fn get_select(&self) -> Vec<(Self, String)> {
    vec![
      (Self::InitUrl,  "init url".into()),
      (Self::Timeout,  "timeout".into()),
      (Self::Style,    "style".into()),
      (Self::Keys,     "keys".into()),
      (Self::SaveFile, "save file".into()),
    ]
  }
}

#[derive(Clone, Debug)]
pub struct User {
  pub timeout:        u64,
  pub save_file:      String,
  pub init_url:       String,
  pub style:          StyleModTable,
  pub keys:           KeysTable,
} 
impl Default for User {
  fn default() -> Self {
    Self {
      timeout:        10,
      init_url:       "gemini://geminiprotocol.net/".into(),
      save_file:      format!("{}/{}", USER_DATA, USER_URLS),
      style:          StyleModTable::default(),
      keys:           KeysTable::default(),
    }
  }
}
impl FromStr for User {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    Self::default().read_table(table)
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
      (UserField::SaveFile, Value::String(v)) => {
        self.save_file = format!("{}/{}", USER_DATA, v);
      }
      (UserField::Timeout, Value::Integer(v)) => {
        self.timeout = u64::try_from(v).map_err(|e| e.to_string())?;
      }
      // read style from another file
      (UserField::Style, Value::String(modname)) => {
        let path   = format!("{}/{}/{}", USER_DATA, USER_STYLES, modname);
        let text   = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let table  = text.parse::<Table>().map_err(|e| e.to_string())?;
        self.style.update_from_table(table)?;
      }
      // read keys from another file
      (UserField::Keys, Value::String(modname)) => {
        let path  = format!("{}/{}/{}", USER_DATA, USER_KEYS, modname);
        let text  = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let table = text.parse::<Table>().map_err(|e| e.to_string())?;
        self.keys.update_from_table(table)?;
      }
      // read style from this file
      (UserField::Style, Value::Table(v)) => {
        self.style.update_from_table(v)?;
      }
      // read keys from this file
      (UserField::Keys, Value::Table(v)) => {
        self.keys.update_from_table(v)?;
      }
      (f, v) => 
        return Err(format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
impl User {
  pub fn get_frame(&self, screen: Rect) -> Frame {
    Frame::new(
        screen, 
        self.style.border.clone(),
        self.style.screen_margin.clone(),
        self.style.text_margin.clone()
      )
      .with_banner_style(*self.style.banner)
      .with_margin_style(*self.style.general)
  }

  pub fn get_styled_gemtext(&self, gemtext: &GemText) -> StyledText {
    let mut text: StyledText = match gemtext.tag {
      GemTag::HeadingOne   => self.style.heading1.into(),
      GemTag::HeadingTwo   => self.style.heading2.into(),
      GemTag::HeadingThree => self.style.heading3.into(),
      GemTag::Text         => self.style.text.into(),
      GemTag::PreFormat    => self.style.preformat.into(),
      GemTag::Link(_, _)   => self.style.link.into(),
      GemTag::BadLink(_)   => self.style.error.into(),
      GemTag::ListItem     => self.style.list.into(),
      GemTag::Quote        => self.style.quote.into(),
    };
    text.with_text(&gemtext.to_string())
  }
}
