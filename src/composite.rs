// src/composite.rs

use crate::{
  common as c,
  user::User,
  screen::Rect,
  text::{StyledText, Style}, 
  widget::{Frame, TextBox, Dynamo, EditBox},
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
  Ack(Dynamo),
  Ask(Dynamo),
  Text(EditBox),
  Select(Dynamo),
}
pub struct Dialog {
  pub prompt:   Dynamo,
  pub response: Response,
} 
impl Dialog {
  pub fn select(prompt: &str, input: Vec<String>, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = Dynamo::new(vec![ptext], &rect.clone().crop_south(2));
    let rtext = input.iter().map(|s| StyledText::from(s.as_str()).with_style(&style));
    let rbox  = Dynamo::new(rtext.collect(), &rect.clone().crop_north(pbox.used.h));
    Dialog {
      prompt:   pbox,
      response: Response::Select(rbox),
    }
  }
  pub fn text(prompt: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = Dynamo::new(vec![ptext], &rect.clone().crop_south(2));
    let rbox  = EditBox::new(&pbox.used.bottom_row()).with_style(&style);
    Dialog {
      prompt:   pbox,
      response: Response::Text(rbox),
    }
  }
  pub fn ask(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = Dynamo::new(vec![ptext], &rect.clone().crop_south(2));
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = Dynamo::new(vec![rtext], &pbox.used.bottom_row());
    Dialog {
      prompt:   pbox,
      response: Response::Ask(rbox),
    }
  }
  pub fn ack(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = Dynamo::new(vec![ptext], &rect.clone().crop_south(2));
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = Dynamo::new(vec![rtext], &pbox.used.bottom_row());
    Dialog {
      prompt:   pbox,
      response: Response::Ack(rbox),
    }
  }
}
