// src/user.rs

use crate::{
    KeyConfig,
    StyleConfig,
    util,
};



#[derive(Debug)]
pub enum ValueErr {
    InvalidTomlType(toml::Value),
    InvalidParse(String),
    Message(String),
}

impl std::fmt::Display for ValueErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidTomlType(t)  => write!(f, "{t:?} is not a valid TOML type"),
            Self::InvalidParse(s) => write!(f, "Failed to parse value. {s}"),
            Self::Message(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug)]
pub struct AssignErr(pub String, pub ValueErr);

impl std::fmt::Display for AssignErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Self(field, value_err) = self;
        match value_err {
            ValueErr::InvalidTomlType(_)  => write!(f, "{field}: '{value_err}'"),
            ValueErr::InvalidParse(_) => write!(f, "{field}: '{value_err}'"),
            ValueErr::Message(e) => write!(f, "{field} > {e}"),
        }
    }
}

impl AssignErr {
    pub fn new<F: std::fmt::Display>(field: F, value: ValueErr) -> Self {
        Self(field.to_string(), value)
    }
}

pub type ValueResult<T> = std::result::Result<T, ValueErr>;

pub type AssignResult = std::result::Result<(), AssignErr>;

pub trait UserAssign<C> {
    type Field;

    // C is context provided by caller
    fn assign(&mut self, _: &Self::Field, _: toml::Value, _: &C)
        -> AssignResult;

    // default to empty implementation
    fn load_context(&mut self, _: &mut toml::Table) {}
}

pub trait UserTable<C>: Sized {

    // always return an instance, collecting all errors encountered
    // into one large error
    fn from_table(_: toml::Table, _: &C) -> (Self, Result<(), String>);

    // same idea as `from_table`
    fn from_str(_: &str, _: &C) -> (Self, Result<(), String>);

    // update all valid assignments, return an error if any assignment
    // returned an error
    fn update_from_table(&mut self, _: toml::Table, _: &C) 
        -> Result<(), String>;

    // same idea as `update_from_table`
    fn update_from_str(&mut self, _: &str, _: &C) -> Result<(), String>;
}


impl<T, F, C> UserTable<C> for T
where   T: UserAssign<C, Field = F> + Default,
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
            && let _ = user_table.assign(&field, value, context)
                .inspect_err(|e| errors.push_str(&format!("{e}"))) {}
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
            let _ = user_table.update_from_table(table, ctx)
                .inspect_err(|e| errors.push_str(&e));
        }
        if errors.len() > 0 {
            (user_table, Err(errors))
        } else {
            (user_table, Ok(()))
        }
    }

    fn update_from_table(&mut self, mut table: toml::Table, context: &C) 
        -> Result<(), String> 
    {
        self.load_context(&mut table);
        let mut errors = "".to_string();

        for (key, value) in table.into_iter() {
            if let Ok(field) = F::from_str(&key)
                .inspect_err(|e| errors.push_str(&e)) 
            && let _ = self.assign(&field, value, context)
                .inspect_err(|e| errors.push_str(&e.to_string())) {}
        }
        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn update_from_str(&mut self, s: &str, ctx: &C) -> Result<(), String> {  
        let mut table = s.parse::<toml::Table>()
            .map_err(|e| e.to_string())?;
        self.load_context(&mut table);
        self.update_from_table(table, ctx)
    }
}


#[derive(Debug)]
pub enum UserConfigField {
    InitUrl, 
    SaveFile,
    Timeout, 
    Style, 
    Keys,
}

impl std::str::FromStr for UserConfigField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init_url"  => Ok(Self::InitUrl),
            "timeout"   => Ok(Self::Timeout),
            "style"     => Ok(Self::Style),
            "keys"      => Ok(Self::Keys),
            "gsave" | 
            "save_file" => Ok(Self::SaveFile),
            s => Err(format!("No field {s} in User table")),
        }
    }
}

impl std::fmt::Display for UserConfigField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InitUrl  => write!(f, "init_url"),
            Self::Timeout  => write!(f, "timeout"),
            Self::Style    => write!(f, "style"),
            Self::Keys     => write!(f, "keys"),
            Self::SaveFile => write!(f, "save_file"),
        }
    }
}


#[derive(Debug)]
pub struct UserConfig {
    pub timeout: u64,
    pub save_file: String,
    pub init_url: String,
    pub style: StyleConfig,
    pub keys: KeyConfig,
    pub urls: Vec<String>,
} 

impl Default for UserConfig {
    fn default() -> Self {
        let urls: Vec<String> = 
            match std::fs::read_to_string(&util::SAVE_FILE) 
        {
            Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
            Err(_) => vec![],
        };
        Self {
            timeout:        10,
            init_url:       "gemini://geminiprotocol.net/".into(),
            save_file:      util::SAVE_FILE.into(),
            style:          StyleConfig::default(),
            keys:           KeyConfig::default(),
            urls,
        }
    }
}

impl UserAssign<()> for UserConfig {
    type Field = UserConfigField;
    fn assign(&mut self, field: &Self::Field, value: toml::Value, ctx: &()) 
        -> AssignResult 
    {
        use toml::Value;
        match (field, value) {
            (UserConfigField::InitUrl, Value::String(value)) => {
                self.init_url = value.into();
            }
            (UserConfigField::SaveFile, Value::String(value)) => {
                self.save_file = format!("{}/{value}", util::DATA_PATH);
            }
            (UserConfigField::Timeout, Value::Integer(value)) => {
                self.timeout = u64::try_from(value)
                    .map_err(|e| 
                        AssignErr::new(
                            field, ValueErr::InvalidParse(e.to_string())
                    ))?;
            }
            // read style from another file
            (UserConfigField::Style, Value::String(string)) => {
                let string = &std::fs::read_to_string(util::get_styles_file(&string))
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
                self.style.update_from_str(string, ctx)
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
            }
            // read style from this file
            (UserConfigField::Style, Value::Table(value)) => {
                self.style.update_from_table(value, ctx)
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
            }
            // read keys from another file
            (UserConfigField::Keys, Value::String(string)) => {
                let string = &std::fs::read_to_string(util::get_keys_file(&string))
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
                self.keys.update_from_str(string, ctx)
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
            }
            // read keys from this file
            (UserConfigField::Keys, Value::Table(value)) => {
                self.keys.update_from_table(value, ctx)
                    .map_err(|e| AssignErr::new(
                        field, ValueErr::Message(e.to_string())
                    ))?;
            }
            (field, value) => return Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            ))
        }
        Ok(())
    }
}

impl UserConfig {
    // may fail when saving a URL or writing to the URL file (2 points)
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
