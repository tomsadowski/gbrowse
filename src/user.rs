// src/user.rs

use crate::{
  SystemControlParams,
  SystemStyleParams,
  Frame, 
  FrameParams,
  GemTag, 
  GemText,
  TextParams,
  Rect,
  DialogParams,
  constants::*,
};


pub trait Assign {
  type Field;

  fn assign(&mut self, _: Self::Field, _: toml::Value) -> Result<(), String>;
}


pub trait UserTable: Sized {
  fn read_table(self, _: toml::Table) -> Result<Self, String>;

  fn update_from_table(&mut self, _: toml::Table) -> Result<(), String>;

  fn update_from_str(&mut self, _: &str) -> Result<(), String>;
}


impl<T, F> UserTable for T
where T: Assign<Field = F>,
      F: std::str::FromStr<Err = String>
{
  fn read_table(mut self, table: toml::Table) -> Result<Self, String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.assign(field, value)?;
    }
    Ok(self)
  }


  fn update_from_table(&mut self, table: toml::Table) -> Result<(), String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.assign(field, value)?;
    }
    Ok(())
  }


  fn update_from_str(&mut self, s: &str) -> Result<(), String> {  
    let table = s.parse::<toml::Table>().map_err(|e| e.to_string())?;
    self.update_from_table(table)?;
    Ok(())
  }
}


pub fn user_from_str<T: UserTable + Default>(s: &str) -> Result<T, String> {
  let table = s.parse::<toml::Table>().map_err(|e| e.to_string())?;
  T::default().read_table(table)
}


pub fn get_init_file(name: &str) -> String {
  format!("{DATA_PATH}/{name}")
}


pub fn get_keys_file(name: &str) -> String {
  format!("{KEYS_PATH}/{name}")
}


pub fn get_styles_file(name: &str) -> String {
  format!("{STYLES_PATH}/{name}")
}


#[derive(Debug)]
pub struct SystemParams {
  pub timeout: u64,
  pub save_file: String,
  pub init_url: String,
  pub style: SystemStyleParams,
  pub keys: SystemControlParams,
  pub urls: Vec<String>,
} 


impl Default for SystemParams {
  fn default() -> Self {
    let urls: Vec<String> = match std::fs::read_to_string(&SAVE_FILE) {
      Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
      Err(e) => vec![],
    };
    Self {
      timeout:        10,
      init_url:       "gemini://geminiprotocol.net/".into(),
      save_file:      SAVE_FILE.into(),
      style:          SystemStyleParams::default(),
      keys:           SystemControlParams::default(),
      urls,
    }
  }
}


impl Assign for SystemParams {
  type Field = UserField;

  fn assign(&mut self, f: Self::Field, v: toml::Value) -> Result<(), String> {
    use toml::Value;
    match (f, v) {
      (UserField::InitUrl, Value::String(v)) => {
        self.init_url = v.into();
      }
      (UserField::SaveFile, Value::String(v)) => {
        self.save_file = format!("{DATA_PATH}/{v}");
      }
      (UserField::Timeout, Value::Integer(v)) => {
        self.timeout = u64::try_from(v).map_err(|e| e.to_string())?;
      }
      // read style from another file
      (UserField::Style, Value::String(v)) => {
        self.style.update_from_str(&std::fs::
          read_to_string(get_styles_file(&v)).map_err(|e| e.to_string())?
        )?;
      }
      // read style from this file
      (UserField::Style, Value::Table(v)) => {
        self.style.update_from_table(v)?;
      }
      // read keys from another file
      (UserField::Keys, Value::String(v)) => {
        self.keys.update_from_str(&std::fs::
          read_to_string(get_keys_file(&v)).map_err(|e| e.to_string())?
        )?;
      }
      // read keys from this file
      (UserField::Keys, Value::Table(v)) => {
        self.keys.update_from_table(v)?;
      }
      (f, v) => return Err(
        format!("field {f:?} value {v:?} not valid here")
      )
    }
    Ok(())
  }
}


impl SystemParams {
  pub fn dlg<'a>(&'a self, prompt: &str) -> DialogParams<'a> {
    DialogParams::from(self).prompt(prompt)
  }


  pub fn save_url(&mut self, url: &url::Url) -> Result<(), String> {
    let url_str = url.to_string();
    if self.urls.iter().any(|url| **url == url_str) {
      Err(format!("URL {url_str} already saved"))
    } else {
      self.urls.push(url_str.clone());
      match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&self.save_file) 
      {
        Err(e) => Err(
          format!("could not create save file: {e}")
        ),
        Ok(mut f) => {
          use std::io::Write;
          for url in self.urls.iter() {
            f.write(&format!("{url}\n").as_bytes());
          }
          Ok(())
        }
      }
    }
  }
}


#[derive(Debug)]
pub enum UserField {
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
      "init_url"  => Ok(Self::InitUrl),
      "timeout"   => Ok(Self::Timeout),
      "style"     => Ok(Self::Style),
      "keys"      => Ok(Self::Keys),
      "gsave" | "save_file" => Ok(Self::SaveFile),
      s => Err(format!("No field {s} in User table")),
    }
  }
}


impl ToString for UserField {
  fn to_string(&self) -> String {
    match self {
     Self::InitUrl  => "init_url".into(),
     Self::Timeout  => "timeout".into(),
     Self::Style    => "style".into(),
     Self::Keys     => "keys".into(),
     Self::SaveFile => "save_file".into(),
    }
  }
}


impl UserField {
  pub fn get_select(&self) -> Vec<(Self, String)> {
    vec![
      (Self::InitUrl,  "init_url".into()),
      (Self::Timeout,  "timeout".into()),
      (Self::Style,    "style".into()),
      (Self::Keys,     "keys".into()),
      (Self::SaveFile, "save_file".into()),
    ]
  }
}
