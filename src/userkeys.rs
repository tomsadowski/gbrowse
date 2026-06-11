// src/userkeys.rs

use crate::{
  Assign,
  Dialog, 
  DlgInput,
  Action,
};
use crossterm::event::KeyCode;
use toml::Value;


#[derive(Copy, Clone, Debug)]
pub struct UserKeys {
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

impl Default for UserKeys {
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

impl Assign for UserKeys {
  type Field = Action;
  fn assign(&mut self, f: Self::Field, v: Value) -> Result<(), String> {
    let get_keycode = || -> Result<KeyCode, String> {
      if let Value::String(s) = v {
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
          s => s
            .chars()
            .next()
            .map(KeyCode::Char)
            .ok_or("could not parse keycode from string".into()),
        }
      } else {
        Err("could not parse keycode from value".into())
      }
    };
    let v = get_keycode()?;
    match f {
      Action::LoadUrl    => self.load_url    = v,
      Action::SaveUrl    => self.save_url    = v,
      Action::Menu       => self.menu        = v,
      Action::MoveUp     => self.up          = v,
      Action::MoveDown   => self.down        = v,
      Action::MoveLeft   => self.left        = v,
      Action::MoveRight  => self.right       = v,
      Action::CycleLeft  => self.cycle_left  = v,
      Action::CycleRight => self.cycle_right = v,
      Action::DelTab     => self.delete_tab  = v,
      Action::NewTab     => self.new_tab     = v,
      Action::Select     => self.select      = v,
      Action::Yes        => self.yes         = v,
      Action::No         => self.no          = v,
      Action::Cancel     => self.cancel      = v,
      Action::Top        => self.top         = v,
      Action::Bottom     => self.bottom      = v,
      Action::PageUp     => self.pgup        = v,
      Action::PageDown   => self.pgdown      = v,
      _ => {},
    }
    Ok(())
  }
}

impl UserKeys {
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
    match dialog.input {
      DlgInput::Ack(_)    => self.get_ack_dialog_action(kc),
      DlgInput::Ask(_)    => self.get_ask_dialog_action(kc),
      DlgInput::Edit(_)   => self.get_edit_dialog_action(kc),
      DlgInput::Select(_) => self.get_select_dialog_action(kc),
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
