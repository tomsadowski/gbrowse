// src/message.rs

use crate::{
  dialog::{Response, Dialog},
};
use url::Url;

#[derive(Clone, Debug)]
pub enum UserAction {
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
pub enum DialogTask {
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
pub enum ResponseType {
  Ack, Ask, Text, Select,
}
#[derive(Clone, Debug)]
pub enum Message {
  Quit,
  Default, 
  MakeAck(DialogTask, String),
  MakeAsk(DialogTask, String),
  MakeText(DialogTask, String),
  MakeSelect(DialogTask, String),
  Action(UserAction),
  DialogTask(DialogTask),
  Resize(u16, u16),
  RequestHere(Url),
  RequestNew(Url),
}
pub enum Focus {
  Tab, 
  Dialog(DialogTask, Dialog),
}

