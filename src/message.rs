// src/message.rs

use crate::{
  dialog::{Dialog},
  widget::{TextBox, EditBox},
};
use url::Url;

#[derive(Clone, Debug)]
pub enum Action {
  // editor
  Insert(char),
  Backspace,
  Enter,
  Delete,
  // MoveLeft,
  // MoveRight,

  // tab
  LoadUrl,
  SaveUrl,
  DelTab, 
  NewTab, 
  HelpView, 
  LogView,
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
  Inspect, 

  // dialog
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl Action {
  pub fn use_editbox(&self, editbox: &mut EditBox) {
    match self {
      Action::Backspace => {editbox.backspace();}
      Action::Delete    => {editbox.delete();}
      Action::Insert(c) => {editbox.insert(*c);}
      Action::MoveLeft  => {editbox.left(1);}
      Action::MoveRight => {editbox.right(1);}
      _ => {}
    }
  }
  pub fn use_textbox(&self, textbox: &mut TextBox) {
    match self {
      Action::PageDown  => {textbox.down(usize::from(textbox.rect.h));}
      Action::PageUp    => {textbox.up(usize::from(textbox.rect.h));}
      Action::Bottom    => {textbox.down(textbox.y_len());}
      Action::Top       => {textbox.up(textbox.y_len());}
      Action::MoveDown  => {textbox.down(1);}
      Action::MoveUp    => {textbox.up(1);}
      Action::MoveLeft  => {textbox.left(1);}
      Action::MoveRight => {textbox.right(1);}
      _ => {}
    }
  }
}
#[derive(Clone, Debug)]
pub enum Task {
  Default, 
  Reply,
  NewTab,
  DelTab,
  LoadUrl,
  Input(String),
  Redirect(String),
  Go(String), 
}
#[derive(Clone, Debug)]
pub enum Message {
  Quit,
  Default, 
  Action(Action),
  Resize(u16, u16),
}
pub enum Focus {
  Tab, 
  Dialog(Task, Dialog),
}
