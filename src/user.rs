// src/user.rs

use crate::{
    SystemControlParams,
    SystemStyleParams,
    DialogParams,
    constants::*,
};


pub trait Assign<C> {
    type Field;

    fn assign(&mut self, _: Self::Field, _: toml::Value, _: &C)
        -> Result<(), String>;

    fn load_context(&mut self, _: &mut toml::Table) {}
}


pub trait UserTable<C>: Sized {
    fn from_table(_: toml::Table, _: &C) -> (Self, Result<(), String>);

    fn from_str(_: &str, _: &C) -> (Self, Result<(), String>);

    fn update_from_table(&mut self, _: toml::Table, _: &C) 
        -> Result<(), String>;

    fn update_from_str(&mut self, _: &str, _: &C) -> Result<(), String>;
}


impl<T, F, C> UserTable<C> for T
where   T: Assign<C, Field = F> + Default,
        F: std::str::FromStr<Err = String>
{
    fn from_table(mut table: toml::Table, context: &C) 
        -> (Self, Result<(), String>) 
    {
        let mut user_table = Self::default();
        user_table.load_context(&mut table);
        let mut errors = String::new();
        for (key, value) in table.into_iter() {
            if let Ok(field) = F::from_str(&key)
                .inspect_err(|e| errors.push_str(&e)) 
            && let _ = user_table.assign(field, value, context)
                .inspect_err(|e| errors.push_str(&e)) {}
        }
        if errors.len() > 0 {
            (user_table, Err(errors))
        } else {
            (user_table, Ok(()))
        }
    }

    fn from_str(s: &str, ctx: &C) -> (Self, Result<(), String>) {  
        let mut user_table = Self::default();
        let mut errors = String::new();
        if let Ok(mut table) = s.parse::<toml::Table>()
            .inspect_err(|e| errors.push_str(&e.to_string()))
        {
            user_table.load_context(&mut table);
            user_table.update_from_table(table, ctx);
        }
        if errors.len() > 0 {
            (user_table, Err(errors))
        } else {
            (user_table, Ok(()))
        }
    }

    fn update_from_table(
        &mut self, mut table: toml::Table, context: &C
    ) -> Result<(), String> {
        self.load_context(&mut table);
        let mut errors = "".to_string();
        for (key, value) in table.into_iter() {
            if let Ok(field) = F::from_str(&key)
                .inspect_err(|e| errors.push_str(&e)) 
            && let _ = self.assign(field, value, context)
                .inspect_err(|e| errors.push_str(&e)) {}
        }
        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn update_from_str(&mut self, s: &str, ctx: &C) -> Result<(), String> {  
        let mut table = s.parse::<toml::Table>().map_err(|e| e.to_string())?;
        self.load_context(&mut table);
        self.update_from_table(table, ctx);
        Ok(())
    }
}


pub fn get_init_file(f: &str) -> String {
    format!("{DATA_PATH}/{f}")
}

pub fn get_keys_file(f: &str) -> String {
    format!("{KEYS_PATH}/{f}")
}

pub fn get_styles_file(f: &str) -> String {
    format!("{STYLES_PATH}/{f}")
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
            Err(_) => vec![],
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


impl Assign<()> for SystemParams {
    type Field = UserField;

    fn assign(&mut self, f: Self::Field, v: toml::Value, ctx: &()) 
        -> Result<(), String> 
    {
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
                    read_to_string(
                        get_styles_file(&v)).map_err(|e| e.to_string()
                    )?,
                    ctx
                )?;
            }
            // read style from this file
            (UserField::Style, Value::Table(v)) => {
                self.style.update_from_table(v, ctx)?;
            }
            // read keys from another file
            (UserField::Keys, Value::String(v)) => {
                self.keys.update_from_str(&std::fs::
                    read_to_string(
                        get_keys_file(&v)).map_err(|e| e.to_string()
                    )?,
                    ctx
                )?;
            }
            // read keys from this file
            (UserField::Keys, Value::Table(v)) => {
                self.keys.update_from_table(v, ctx)?;
            }
            (f, v) => return Err(
                format!("field {f:?} value {v:?} not valid here")
            )
        }
        Ok(())
    }
}


impl SystemParams {

    // convenience method
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
                        let _ = f.write(&format!("{url}\n").as_bytes());
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
