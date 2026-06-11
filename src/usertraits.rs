// src/user_traits.rs

use toml::{Table, Value};

pub trait Assign {
  type Field;

  fn assign(&mut self, field: Self::Field, value: Value) 
    -> Result<(), String>;
}

pub trait UserTable: Sized {
  fn read_table(self, table: Table) 
    -> Result<Self, String>;

  fn update_from_table(&mut self, table: Table) 
    -> Result<(), String>;

  fn update_from_str(&mut self, s: &str) 
    -> Result<(), String>;
}

impl<T, F> UserTable for T
where 
  T: Assign<Field = F>,
  F: std::str::FromStr<Err = String>
{
  fn read_table(mut self, table: Table) -> Result<Self, String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.assign(field, value)?;
    }
    Ok(self)
  }

  fn update_from_table(&mut self, table: Table) -> Result<(), String> {
    for (key, value) in table.into_iter() {
      let field = F::from_str(&key)?;
      self.assign(field, value)?;
    }
    Ok(())
  }

  fn update_from_str(&mut self, s: &str) -> Result<(), String> {  
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    self.update_from_table(table)?;
    Ok(())
  }
}

pub trait UserFromStr: Sized {
  fn user_from_str(s: &str) -> Result<Self, String>;
}

impl<T> UserFromStr for T
where T: UserTable + Default,
{
  fn user_from_str(s: &str) -> Result<Self, String> {
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    Self::default().read_table(table)
  }
}
