// src/composite.rs

use crate::{
  common as c,
  user::User,
  screen::Rect,
  text::{StyledText, Style}, 
  widget::{Frame, TextBox, EditBox},
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


pub struct Tab {
  pub url_str: String,
  pub gemdoc:  Option<GemDoc>,
  pub content: TextBox,
} 
impl Deref for Tab {
  type Target = TextBox;
  fn deref(&self) -> &Self::Target {
    &self.content
  }
}
impl DerefMut for Tab {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.content
  }
}
impl Tab {
  pub fn init(rect: &Rect, url_str: &str) -> Self {
    let mut content = TextBox::default();
    content.rect = rect.clone();
    Self {
      content, 
      gemdoc:  None,
      url_str: url_str.into(),
    }
  }
}

pub enum Response {
  Ack(TextBox),
  Ask(TextBox),
  Text(EditBox),
  Select(TextBox),
}
pub struct Dialog {
  pub prompt:   TextBox,
  pub response: Response,
} 
impl Dialog {
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
