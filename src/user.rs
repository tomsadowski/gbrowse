// src/user.rs

use crate::{
  keys::KeysTable,
  style::StyleTable,
  tab::Tab,
  widget::{Frame, TextBox},
  view::Rect,
  text::StyledText,
  gemdoc::{GemText, GemTag},
};
use toml::{Table, Value};
use std::io::Write;


pub const DATA_PATH:   &str = "gdata";
pub const SAVE_FILE:   &str = "gdata/urls";
pub const INIT_FILE:   &str = "gdata/init";
pub const STYLES_PATH: &str = "gdata/styles";
pub const KEYS_PATH:   &str = "gdata/keys";

pub fn get_init_file(name: &str) -> String {
  format!("{}/{}", DATA_PATH, name)
}

pub fn get_keys_file(name: &str) -> String {
  format!("{}/{}", KEYS_PATH, name)
}

pub fn get_styles_file(name: &str) -> String {
  format!("{}/{}", STYLES_PATH, name)
}

pub trait UserTable<F>: Sized 
where 
  F: std::str::FromStr<Err = String> 
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
impl std::str::FromStr for UserField {
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
  pub style:          StyleTable,
  pub keys:           KeysTable,
  pub urls:           Vec<String>,
} 
impl Default for User {
  fn default() -> Self {
    let urls: Vec<String> = match std::fs::read_to_string(&SAVE_FILE) {
      Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
      Err(e) => vec![],
    };
    Self {
      timeout:        10,
      init_url:       "gemini://geminiprotocol.net/".into(),
      save_file:      SAVE_FILE.into(),
      style:          StyleTable::default(),
      keys:           KeysTable::default(),
      urls,
    }
  }
}
impl std::str::FromStr for User {
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
        self.save_file = format!("{}/{}", DATA_PATH, v);
      }
      (UserField::Timeout, Value::Integer(v)) => {
        self.timeout = u64::try_from(v).map_err(|e| e.to_string())?;
      }
      // read style from another file
      (UserField::Style, Value::String(v)) => {
        self.style.update_from_str(
          &std::fs::read_to_string(
            get_styles_file(&v)
          ).map_err(|e| e.to_string())?
        )?;
      }
      // read style from this file
      (UserField::Style, Value::Table(v)) => {
        self.style.update_from_table(v)?;
      }
      // read keys from another file
      (UserField::Keys, Value::String(v)) => {
        self.keys.update_from_str(
          &std::fs::read_to_string(
            get_keys_file(&v)
          ).map_err(|e| e.to_string())?
        )?;
      }
      // read keys from this file
      (UserField::Keys, Value::Table(v)) => {
        self.keys.update_from_table(v)?;
      }
      (f, v) => return Err(
        format!("field {:?} value {:?} not valid here", f, v))
    }
    Ok(())
  }
}
impl User {
  pub fn save_url(&mut self, url: &url::Url) -> Result<(), String> {
    let url_str = url.to_string();
    if self.urls.iter().any(|url| **url == url_str) {
      Err(format!("URL {} already saved", url_str))
    } else {
      self.urls.push(url_str.clone());
      // write to save file
      match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&self.save_file) 
      {
        Err(e) => Err(format!("could not create save file: {}", &e)),
        Ok(mut f) => {
          for url in self.urls.iter() {
            f.write(&format!("{}\n", url).as_bytes());
          }
          Ok(())
        }
      }
    }
  }

  pub fn get_frame(&self, screen: Rect) -> Frame {
    Frame::from(screen)
      .with_screen_margin(self.style.screen_margin)
      .with_text_margin(self.style.text_margin)
      .with_banner_style(self.style.banner)
      .with_footer_style(self.style.banner)
      .with_margin_style(self.style.general)
      .with_border_style(self.style.border)
  }

  pub fn get_styled_gemtext(&self, gemtext: &GemText) -> StyledText {
    let mut text: StyledText = match gemtext.tag {
      GemTag::HeadingOne   => self.style.heading1.into(),
      GemTag::HeadingTwo   => self.style.heading2.into(),
      GemTag::HeadingThree => self.style.heading3.into(),
      GemTag::Text         => self.style.text.into(),
      GemTag::PreFormat    => self.style.preformat.into(),
      GemTag::Link(_)      => self.style.link.into(),
      GemTag::ListItem     => self.style.list.into(),
      GemTag::Quote        => self.style.quote.into(),
    };
    text.with_text(&gemtext.to_string())
  }
}
