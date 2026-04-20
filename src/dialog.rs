// src/dialog.rs

use crate::{
  common as c,
  Message,
  user::User,
  text::{StyledText, Style}, 
  widget::{Rect, Frame, TextBox, EditBox},
  protocol::{GemDoc, GemTag, Status, Scheme, get_data},
};
use crossterm::{
  QueueableCommand,
  cursor::{self, SetCursorStyle},
  terminal::{self, Clear, ClearType},
  event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use url::Url;
use std::{
  fs, thread, env,
  ops::{Deref, DerefMut},
  sync::mpsc,
  time::Duration,
  str::FromStr,
  io::{self, Write, Read, stdout, Stdout},
};


pub enum ResponseType {
  Ack,
  Ask,
  Text,
  Select,
}
pub enum Response {
  Ack(TextBox),
  Ask(TextBox),
  Text(EditBox),
  Select(TextBox),
}
impl Response {
  pub fn get_type(&self) -> ResponseType {
    match self {
      Response::Ack(_) => ResponseType::Ack,
      Response::Ask(_) => ResponseType::Ask,
      Response::Text(_) => ResponseType::Text,
      Response::Select(_) => ResponseType::Select,
    }
  }
}
pub struct Dialog {
  pub prompt:   TextBox,
  pub response: Response,
} 
impl Dialog {
  pub fn resize(&mut self, rect: &Rect) {
    self.prompt.resize(rect);
    match &mut self.response {
      Response::Ack(r) | Response::Ask(r) | Response::Select(r) => 
        r.resize(rect),
      Response::Text(r) => r.resize(rect),
    }
  }
  pub fn select(prompt: &str, input: Vec<String>, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = input.iter().map(|s| StyledText::from(s.as_str()).with_style(&style));
    let rbox  = 
      TextBox::new(rtext.collect(), &rect.cropped_north(pbox.used_rect().h))
        .write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Select(rbox),
    }
  }
  pub fn text(prompt: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rbox  = EditBox::new(&pbox.used_rect().bottom_row()).with_style(&style);
    Dialog {
      prompt:   pbox,
      response: Response::Text(rbox),
    }
  }
  pub fn ask(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = TextBox::new(vec![rtext], &pbox.used_rect().bottom_row()).write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Ask(rbox),
    }
  }
  pub fn ack(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = TextBox::new(vec![rtext], &pbox.used_rect().bottom_row()).write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Ack(rbox),
    }
  }
}
