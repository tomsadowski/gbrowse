// src/keys.rs

use crate::{
  cursor::UnitCursor,
  user::UserTable,
  widget::{EditBox, TextBox},
  dialog::{Dialog, Response},
};
use crossterm::event::KeyCode;
use toml::Value;
use std::str::FromStr;


#[derive(Copy, Clone, Debug)]
pub enum Action {
  // editor
  Insert(char),
  Backspace,
  Enter,
  Delete,
  // tab
  Menu,
  LoadUrl,
  SaveUrl,
  DelTab, 
  NewTab, 
  CycleLeft, 
  CycleRight, 
  // selector
  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  Top,
  Bottom,
  PageUp,
  PageDown,
  Select, 
  // dialog
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl FromStr for Action {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "load_url"    => Ok(Self::LoadUrl),
      "save_url"    => Ok(Self::SaveUrl),
      "move_up"     => Ok(Self::MoveUp),
      "menu"        => Ok(Self::Menu),
      "move_down"   => Ok(Self::MoveDown),
      "move_left"   => Ok(Self::MoveLeft),
      "move_right"  => Ok(Self::MoveRight),
      "cycle_left"  => Ok(Self::CycleLeft),
      "cycle_right" => Ok(Self::CycleRight),
      "delete_tab"  => Ok(Self::DelTab),
      "new_tab"     => Ok(Self::NewTab),
      "select"      => Ok(Self::Select),
      "ack"         => Ok(Self::Ack),
      "yes"         => Ok(Self::Yes),
      "no"          => Ok(Self::No),
      "cancel"      => Ok(Self::Cancel),
      s => 
        Err(format!("Keys table does not contain field {}", s)),
    }
  }
}

#[derive(Copy, Clone, Debug)]
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
  pub select:      KeyCode,
  pub menu:        KeyCode,
  pub load_url:    KeyCode,
  pub save_url:    KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,
  pub yes:         KeyCode, 
  pub no:          KeyCode,
  pub cancel:      KeyCode,
} 
impl Default for KeysTable {
  fn default() -> Self {
    Self {
      load_url:    KeyCode::Char('u'),
      menu:        KeyCode::Char('m'),
      save_url:    KeyCode::Char('U'),
      delete_tab:  KeyCode::Char('d'),
      new_tab:     KeyCode::Char('n'),
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
      select:      KeyCode::Enter,
      yes:         KeyCode::Char('y'), 
      no:          KeyCode::Char('n'),
      cancel:      KeyCode::Esc,
    }
  }
}
impl UserTable<Action> for KeysTable {
  fn try_assign(&mut self, field: Action, value: Value) 
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
          s => s.chars().next().map(KeyCode::Char)
            .ok_or("could not parse keycode from string".into()),
        }
      } else {
        Err("could not parse keycode from value".into())
      }
    };
    let value = get_keycode()?;
    match field {
      Action::LoadUrl    => self.load_url    = value,
      Action::SaveUrl    => self.save_url    = value,
      Action::Menu       => self.menu        = value,
      Action::MoveUp     => self.up          = value,
      Action::MoveDown   => self.down        = value,
      Action::MoveLeft   => self.left        = value,
      Action::MoveRight  => self.right       = value,
      Action::CycleLeft  => self.cycle_left  = value,
      Action::CycleRight => self.cycle_right = value,
      Action::DelTab     => self.delete_tab  = value,
      Action::NewTab     => self.new_tab     = value,
      Action::Select     => self.select      = value,
      Action::Yes        => self.yes         = value,
      Action::No         => self.no          = value,
      Action::Cancel     => self.cancel      = value,
      Action::Top        => self.top         = value,
      Action::Bottom     => self.bottom      = value,
      Action::PageUp     => self.pgup        = value,
      Action::PageDown   => self.pgdown      = value,
      _ => {},
    }
    Ok(())
  }
}
impl KeysTable {
  pub fn get_tab_action(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.load_url    == kc {Some(Action::LoadUrl)
    } else if &self.save_url    == kc {Some(Action::SaveUrl)
    } else if &self.cycle_left  == kc {Some(Action::CycleLeft)
    } else if &self.cycle_right == kc {Some(Action::CycleRight)
    } else if &self.delete_tab  == kc {Some(Action::DelTab)
    } else if &self.new_tab     == kc {Some(Action::NewTab)
    } else if &self.menu        == kc {Some(Action::Menu)
    } else                            {self.get_text_box_action(kc)}
  }

  pub fn get_text_box_action(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.up          == kc {Some(Action::MoveUp)
    } else if &self.down        == kc {Some(Action::MoveDown)
    } else if &self.left        == kc {Some(Action::MoveLeft)
    } else if &self.right       == kc {Some(Action::MoveRight)
    } else if &self.select      == kc {Some(Action::Select)
    } else if &self.top         == kc {Some(Action::Top)
    } else if &self.bottom      == kc {Some(Action::Bottom)
    } else if &self.pgup        == kc {Some(Action::PageUp)  
    } else if &self.pgdown      == kc {Some(Action::PageDown)
    } else {None}
  }

  pub fn get_edit_box_action(&self, kc: &KeyCode) -> Option<Action> {
    match kc {
      KeyCode::Left      => Some(Action::MoveLeft),
      KeyCode::Right     => Some(Action::MoveRight),
      KeyCode::Backspace => Some(Action::Backspace),
      KeyCode::Delete    => Some(Action::Delete),
      KeyCode::Enter     => Some(Action::Enter),
      KeyCode::Char(c)   => Some(Action::Insert(*c)),
      _                  => None,
    }
  }

  pub fn get_dlg_action(&self, dialog: &Dialog, kc: &KeyCode) -> Option<Action> {
    match dialog.response {
      Response::Ack(_)    => self.get_ack_dialog_action(kc),
      Response::Ask(_)    => self.get_ask_dialog_action(kc),
      Response::Edit(_)   => self.get_edit_dialog_action(kc),
      Response::Select(_) => self.get_select_dialog_action(kc),
    }
  }

  pub fn get_ack_dialog_action(&self, kc: &KeyCode) -> Option<Action> {
    Some(Action::Ack)
  }

  pub fn get_ask_dialog_action(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.cancel == kc {Some(Action::Cancel)
    } else if &self.yes    == kc {Some(Action::Yes)
    } else if &self.no     == kc {Some(Action::No)
    } else                       {None}
  }

  pub fn get_select_dialog_action(&self, kc: &KeyCode) -> Option<Action> {
    if &self.cancel == kc {Some(Action::Cancel)
    } else                {self.get_text_box_action(kc)}
  }

  pub fn get_edit_dialog_action(&self, kc: &KeyCode) -> Option<Action> {
    if &self.cancel == kc {Some(Action::Cancel)
    } else                {self.get_edit_box_action(kc)}
  }
}
