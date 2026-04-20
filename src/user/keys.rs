// src/user/keys.rs

use crate::{
  Focus, 
  message::{Message, UserAction},
  user::UserTable,
  dialog::{Dialog, Response, ResponseType},
  widget::{TextBox},
};
use crossterm::event::KeyCode;
use toml::Value;
use std::str::FromStr;

impl FromStr for UserAction {
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
  pub fn get_dialog_action(&self, dialog: &Dialog, kc: &KeyCode) -> Option<UserAction> {
    match dialog.response {
      Response::Ack(_)    => self.get_ack_dialog_action(kc),
      Response::Ask(_)    => self.get_ask_dialog_action(kc),
      Response::Text(_)   => self.get_text_dialog_action(kc),
      Response::Select(_) => self.get_select_dialog_action(kc),
    }
  }
  pub fn get_tab_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if        &self.load_url    == kc {Some(UserAction::LoadUrl)
    } else if &self.help_view   == kc {Some(UserAction::SaveUrl)
    } else if &self.help_view   == kc {Some(UserAction::HelpView)
    } else if &self.log_view    == kc {Some(UserAction::LogView)
    } else if &self.cycle_left  == kc {Some(UserAction::CycleLeft)
    } else if &self.cycle_right == kc {Some(UserAction::CycleRight)
    } else if &self.delete_tab  == kc {Some(UserAction::DelTab)
    } else if &self.new_tab     == kc {Some(UserAction::NewTab)
    } else                            {self.get_text_box_action(kc)}
  }
  pub fn get_text_box_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if        &self.up          == kc {Some(UserAction::MoveUp)
    } else if &self.down        == kc {Some(UserAction::MoveDown)
    } else if &self.left        == kc {Some(UserAction::MoveLeft)
    } else if &self.right       == kc {Some(UserAction::MoveRight)
    } else if &self.inspect     == kc {Some(UserAction::Inspect)
    } else if &self.top         == kc {Some(UserAction::Top)
    } else if &self.bottom      == kc {Some(UserAction::Bottom)
    } else if &self.pgup        == kc {Some(UserAction::PageUp)  
    } else if &self.pgdown      == kc {Some(UserAction::PageDown)
    } else {None}
  }
  pub fn get_ack_dialog_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if        &self.cancel == kc {Some(UserAction::Cancel)
    } else if &self.ack    == kc {Some(UserAction::Ack)
    } else                       {None}
  }
  pub fn get_ask_dialog_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if        &self.cancel == kc {Some(UserAction::Cancel)
    } else if &self.yes    == kc {Some(UserAction::Yes)
    } else if &self.no     == kc {Some(UserAction::No)
    } else                       {None}
  }
  pub fn get_select_dialog_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if &self.cancel == kc {Some(UserAction::Cancel)
    } else                {self.get_text_box_action(kc)}
  }
  pub fn get_text_dialog_action(&self, kc: &KeyCode) -> Option<UserAction> {
    if &self.cancel == kc {Some(UserAction::Cancel)
    } else                {self.get_edit_box_action(kc)}
  }
  pub fn get_edit_box_action(&self, kc: &KeyCode) -> Option<UserAction> {
    match kc {
      KeyCode::Left      => Some(UserAction::MoveLeft),
      KeyCode::Right     => Some(UserAction::MoveRight),
      KeyCode::Backspace => Some(UserAction::Backspace),
      KeyCode::Delete    => Some(UserAction::Delete),
      KeyCode::Enter     => Some(UserAction::Enter),
      KeyCode::Char(c)   => Some(UserAction::Insert(*c)),
      _                  => None,
    }
  }
}
impl UserTable<UserAction> for KeysTable {
  fn try_assign(&mut self, field: UserAction, value: Value) -> Result<(), String> {
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
      UserAction::LoadUrl    => self.load_url    = value,
      UserAction::SaveUrl    => self.save_url    = value,
      UserAction::HelpView   => self.help_view   = value,
      UserAction::LogView    => self.log_view    = value,
      UserAction::MoveUp     => self.up          = value,
      UserAction::MoveDown   => self.down        = value,
      UserAction::MoveLeft   => self.left        = value,
      UserAction::MoveRight  => self.right       = value,
      UserAction::CycleLeft  => self.cycle_left  = value,
      UserAction::CycleRight => self.cycle_right = value,
      UserAction::DelTab     => self.delete_tab  = value,
      UserAction::NewTab     => self.new_tab     = value,
      UserAction::Inspect    => self.inspect     = value,
      UserAction::Ack        => self.ack         = value,
      UserAction::Yes        => self.yes         = value,
      UserAction::No         => self.no          = value,
      UserAction::Cancel     => self.cancel      = value,
      UserAction::Top        => self.top         = value,
      UserAction::Bottom     => self.bottom      = value,
      UserAction::PageUp     => self.pgup        = value,
      UserAction::PageDown   => self.pgdown      = value,
      _ => {},
    }
    Ok(())
  }
}
