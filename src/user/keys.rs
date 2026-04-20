// src/user/keys.rs

use crate::{
  Focus,
  user::UserTable,
  dialog::{Dialog, ResponseType},
  widget::{TextBox},
};
use crossterm::event::KeyCode;
use toml::Value;
use std::str::FromStr;

#[derive(Debug)]
pub enum Action {
  Insert(char),
  Backspace,
  Enter,
  Delete,
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
impl FromStr for Action {
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
impl KeysTable {
  pub fn get(&self, focus: Focus, kc: &KeyCode) -> Option<Action> {
    match focus {
      Focus::Tab            => self.get_tab_key(kc),
      Focus::Dialog(response) => match response {
        ResponseType::Ack    => self.get_ack(kc),
        ResponseType::Ask    => self.get_ask(kc),
        ResponseType::Text   => self.get_text(kc),
        ResponseType::Select => None,
      }
    }
  }
  pub fn get_text_box_key(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.up          == kc {Some(Action::MoveUp)
    } else if &self.down        == kc {Some(Action::MoveDown)
    } else if &self.left        == kc {Some(Action::MoveLeft)
    } else if &self.right       == kc {Some(Action::MoveRight)
    } else if &self.inspect     == kc {Some(Action::Inspect)
    } else if &self.top         == kc {Some(Action::Top)
    } else if &self.bottom      == kc {Some(Action::Bottom)
    } else if &self.pgup        == kc {Some(Action::PageUp)  
    } else if &self.pgdown      == kc {Some(Action::PageDown)
    } else {None}
  }
  pub fn get_tab_key(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.load_url    == kc {Some(Action::LoadUrl)
    } else if &self.help_view   == kc {Some(Action::SaveUrl)
    } else if &self.help_view   == kc {Some(Action::HelpView)
    } else if &self.log_view    == kc {Some(Action::LogView)
    } else if &self.cycle_left  == kc {Some(Action::CycleLeft)
    } else if &self.cycle_right == kc {Some(Action::CycleRight)
    } else if &self.delete_tab  == kc {Some(Action::DelTab)
    } else if &self.new_tab     == kc {Some(Action::NewTab)
    } else                            {self.get_text_box_key(kc)}
  }
  pub fn get_ack(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.cancel == kc {Some(Action::Cancel)
    } else if &self.ack    == kc {Some(Action::Ack)
    } else                       {None}
  }
  pub fn get_ask(&self, kc: &KeyCode) -> Option<Action> {
    if        &self.cancel == kc {Some(Action::Cancel)
    } else if &self.yes    == kc {Some(Action::Yes)
    } else if &self.no     == kc {Some(Action::No)
    } else                       {None}
  }
  pub fn get_select(&self, kc: &KeyCode) -> Option<Action> {
    if &self.cancel == kc {Some(Action::Cancel)
    } else                {self.get_text_box_key(kc)}
  }
  pub fn get_text(&self, kc: &KeyCode) -> Option<Action> {
    if &self.cancel == kc {Some(Action::Cancel)
    } else                {self.get_edit_box(kc)}
  }
  pub fn get_edit_box(&self, kc: &KeyCode) -> Option<Action> {
    match kc {
      KeyCode::Left      => Some(Action::MoveLeft),
      KeyCode::Right     => Some(Action::MoveRight),
      KeyCode::Backspace => Some(Action::Backspace),
      KeyCode::Delete    => Some(Action::Delete),
      KeyCode::Char(c)   => Some(Action::Insert(*c)),
      _                  => None,
    }
  }
}
impl UserTable<Action> for KeysTable {
  fn try_assign(&mut self, field: Action, value: Value) -> Result<(), String> {
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
      Action::LoadUrl    => self.load_url    = value,
      Action::SaveUrl    => self.save_url    = value,
      Action::HelpView   => self.help_view   = value,
      Action::LogView    => self.log_view    = value,
      Action::MoveUp     => self.up          = value,
      Action::MoveDown   => self.down        = value,
      Action::MoveLeft   => self.left        = value,
      Action::MoveRight  => self.right       = value,
      Action::CycleLeft  => self.cycle_left  = value,
      Action::CycleRight => self.cycle_right = value,
      Action::DelTab     => self.delete_tab  = value,
      Action::NewTab     => self.new_tab     = value,
      Action::Inspect    => self.inspect     = value,
      Action::Ack        => self.ack         = value,
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
