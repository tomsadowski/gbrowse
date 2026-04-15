// src/user/keys.rs

use crate::user::UserTable;
use crossterm::event::KeyCode;
use toml::Value;
use std::str::FromStr;

#[derive(Debug)]
pub enum KeysField {
  LoadUser,
  TabView, 
  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  CycleLeft, 
  CycleRight, 
  DelTab, 
  NewTab, 
  Inspect, 
  Ack, 
  Yes, 
  No, 
  Cancel,
  PageUp,
  PageDown,
  Top,
  Bottom,
}
impl FromStr for KeysField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "tab_view"    => Ok(Self::TabView),
      "load_user"   => Ok(Self::LoadUser),
      "move_up"     => Ok(Self::MoveUp),
      "move_down"   => Ok(Self::MoveDown),
      "move_left"   => Ok(Self::MoveLeft),
      "move_right"  => Ok(Self::MoveRight),
      "cycle_left"  => Ok(Self::CycleLeft),
      "cycle_right" => Ok(Self::CycleRight),
      "delete_tab"  => Ok(Self::DelTab),
      "new_tab"     => Ok(Self::NewTab),
      "inspect"     => Ok(Self::Inspect),
      "ack"         => Ok(Self::Ack),
      "yes"         => Ok(Self::Yes),
      "no"          => Ok(Self::No),
      "cancel"      => Ok(Self::Cancel),
      s => Err(format!("Keys table does not contain field {}", s)),
    }
  }
}
#[derive(Clone)]
pub struct KeysTable {
  pub cancel:      KeyCode,
  pub load_user:   KeyCode,
  pub tab_view:    KeyCode,
  pub up:          KeyCode,
  pub down:        KeyCode,
  pub left:        KeyCode,
  pub right:       KeyCode,
  pub cycle_left:  KeyCode,
  pub cycle_right: KeyCode,
  pub inspect:     KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,
  pub ack:         KeyCode, 
  pub yes:         KeyCode, 
  pub no:          KeyCode,
  pub top:         KeyCode,
  pub bottom:      KeyCode,
  pub pgup:        KeyCode,
  pub pgdown:      KeyCode,
} 
impl Default for KeysTable {
  fn default() -> Self {
    Self {
      cancel:      KeyCode::Esc,
      load_user:   KeyCode::Char('c'),
      tab_view:    KeyCode::Char('t'),
      up:          KeyCode::Up,
      down:        KeyCode::Down,
      left:        KeyCode::Left,
      right:       KeyCode::Right,
      cycle_left:  KeyCode::Char('E'),
      cycle_right: KeyCode::Char('N'),
      inspect:     KeyCode::Enter,
      delete_tab:  KeyCode::Char('d'),
      new_tab:     KeyCode::Char('n'),
      ack:         KeyCode::Enter, 
      yes:         KeyCode::Char('y'), 
      no:          KeyCode::Char('n'),
      top:         KeyCode::Home,
      bottom:      KeyCode::End,
      pgup:        KeyCode::PageUp,
      pgdown:      KeyCode::PageDown,
    }
  }
}
impl UserTable<KeysField> for KeysTable {
  fn try_assign(&mut self, field: KeysField, value: Value) 
    -> Result<(), String> 
  {
    let get_keycode = || -> Result<KeyCode, String> {
      if let Value::String(s) = value {
        match s.as_str() {
          "esc" | "escape" => Ok(KeyCode::Esc),
          "ent" | "enter"  => Ok(KeyCode::Enter),
          "space"          => Ok(KeyCode::Char(' ')),
          "left"           => Ok(KeyCode::Left),
          "up"             => Ok(KeyCode::Up),
          "down"           => Ok(KeyCode::Down),
          "right"          => Ok(KeyCode::Right),
          "pgdown"         => Ok(KeyCode::PageDown),
          "pgup"           => Ok(KeyCode::PageUp),
          "end"            => Ok(KeyCode::End),
          "home"           => Ok(KeyCode::Home),
          s => 
            s.chars().next().map(|c| KeyCode::Char(c))
            .ok_or("could not parse keycode from string".into()),
        }
      } else {
        Err("could not parse keycode from value".into())
      }
    };
    let value = get_keycode()?;
    match field {
      KeysField::LoadUser   => self.load_user   = value,
      KeysField::TabView    => self.tab_view    = value,
      KeysField::MoveUp     => self.up          = value,
      KeysField::MoveDown   => self.down        = value,
      KeysField::MoveLeft   => self.left        = value,
      KeysField::MoveRight  => self.right       = value,
      KeysField::CycleLeft  => self.cycle_left  = value,
      KeysField::CycleRight => self.cycle_right = value,
      KeysField::DelTab     => self.delete_tab  = value,
      KeysField::NewTab     => self.new_tab     = value,
      KeysField::Inspect    => self.inspect     = value,
      KeysField::Ack        => self.ack         = value,
      KeysField::Yes        => self.yes         = value,
      KeysField::No         => self.no          = value,
      KeysField::Cancel     => self.cancel      = value,
      KeysField::Top        => self.top         = value,
      KeysField::Bottom     => self.bottom      = value,
      KeysField::PageUp     => self.pgup        = value,
      KeysField::PageDown   => self.pgdown      = value,
    }
    Ok(())
  }
}
