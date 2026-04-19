// src/user/keys.rs

use crate::{
  user::UserTable,
  widget::{TextBox},
};
use crossterm::event::KeyCode;
use toml::Value;
use std::str::FromStr;


pub trait ProcessKeycode {
}

#[derive(Debug)]
pub enum KeysField {
  LoadUrl,
  SaveUrl,
  DelTab, 
  NewTab, 
  HelpView, 
  LogView,

  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  PageUp,
  PageDown,
  Top,
  Bottom,
  CycleLeft, 
  CycleRight, 

  Inspect, 
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl FromStr for KeysField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "help_view"   => Ok(Self::HelpView),
      "log_view"    => Ok(Self::LogView),
      "load_url"    => Ok(Self::LoadUrl),
      "save_url"    => Ok(Self::SaveUrl),
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
#[derive(Clone, Debug)]
pub struct KeysTable {
  pub up:          KeyCode,
  pub down:        KeyCode,
  pub left:        KeyCode,
  pub right:       KeyCode,
  pub top:         KeyCode,
  pub bottom:      KeyCode,
  pub pgup:        KeyCode,
  pub pgdown:      KeyCode,
  pub cycle_left:  KeyCode,
  pub cycle_right: KeyCode,

  pub load_url:    KeyCode,
  pub save_url:    KeyCode,
  pub help_view:   KeyCode,
  pub log_view:    KeyCode,
  pub inspect:     KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,

  pub ack:         KeyCode, 
  pub yes:         KeyCode, 
  pub no:          KeyCode,
  pub cancel:      KeyCode,
} 
impl KeysTable {
  pub fn move_content(&self, kc: &KeyCode, content: &mut TextBox) -> bool {
    if kc == &self.pgdown {
      content.down(usize::from(content.rect.h))
    } else if kc == &self.pgup {
      content.up(usize::from(content.rect.h))
    } else if kc == &self.bottom {
      content.down(content.y_len())
    } else if kc == &self.top {
      content.up(content.y_len())
    } else if kc == &self.down {
      content.down(1)
    } else if kc == &self.up {
      content.up(1)
    } else if kc == &self.left {
      content.left(1)
    } else if kc == &self.right {
      content.right(1)
    } else {false}
  }
}
impl Default for KeysTable {
  fn default() -> Self {
    Self {
      load_url:    KeyCode::Char('u'),
      save_url:    KeyCode::Char('U'),
      delete_tab:  KeyCode::Char('d'),
      new_tab:     KeyCode::Char('n'),
      help_view:   KeyCode::Char('h'),
      log_view:    KeyCode::Char('l'),
      up:          KeyCode::Up,
      down:        KeyCode::Down,
      left:        KeyCode::Left,
      right:       KeyCode::Right,
      top:         KeyCode::Home,
      bottom:      KeyCode::End,
      pgup:        KeyCode::PageUp,
      pgdown:      KeyCode::PageDown,
      cycle_left:  KeyCode::Char(','),
      cycle_right: KeyCode::Char('.'),
      inspect:     KeyCode::Enter,
      ack:         KeyCode::Enter, 
      yes:         KeyCode::Char('y'), 
      no:          KeyCode::Char('n'),
      cancel:      KeyCode::Esc,
    }
  }
}
impl UserTable<KeysField> for KeysTable {
  fn try_assign(&mut self, field: KeysField, value: Value) -> Result<(), String> {
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
      KeysField::LoadUrl    => self.load_url    = value,
      KeysField::SaveUrl    => self.save_url    = value,
      KeysField::HelpView   => self.help_view   = value,
      KeysField::LogView    => self.log_view    = value,
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
