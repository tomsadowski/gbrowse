// src/message.rs

use crate::{
  dialog::{Response, Dialog},
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
  RequestHere(Url),
  RequestNew(Url),
}
pub enum Focus {
  Tab, 
  Dialog(Task, Dialog),
}

