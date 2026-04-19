// src/tab.rs

use crate::{
  common as c,
  Message,
  user::User,
  text::{StyledText, Style, Linear}, 
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


pub struct TabList {
  pub head:      usize,
  pub tabs:      Vec<Tab>,
  pub pending:   bool,
} 
impl Linear for TabList {
  fn len(&self) -> usize {
    self.tabs.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
}
impl Deref for TabList {
  type Target = Tab;
  fn deref(&self) -> &Self::Target {
    &self.tabs[self.head]
  }
}
impl DerefMut for TabList {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.tabs[self.head]
  }
}
impl TabList {
  // maybe return bool
  pub fn add(&mut self, url_str: &str) {
    // search for tab with same url_str
    let search = self.tabs.iter_mut().enumerate().find(|(_, tab)| tab.url_str == url_str);
    // move head to location of tab with url_str
    if let Some((idx, _)) = search {
      self.head = idx;
    // or make a new tab
    } else {
      let new_tab = Tab::init(&self.rect, &url_str);
      if self.head + 1 == self.tabs.len() {
        self.tabs.push(new_tab);
        self.head += 1;
      }
      else {
        self.head += 1;
        self.tabs.insert(self.head, new_tab);
      }
    }
    self.reset_state();
  }
  pub fn delete(&mut self) {
    if self.tabs.len() > 1 {
      self.tabs.remove(self.head);
      self.wrapping_backward(1);
    }
  }
  pub fn new(tab: Tab) -> Self {
    Self {tabs: vec![tab], head: 0, pending: false}
  }
  pub fn resize(&mut self, rect: &Rect) {
    for tab in self.tabs.iter_mut() {
      tab.resize(rect);
    }
  }
  pub fn banner_text(&self) -> String {
    let text = format!(" {}/{} - {} ", self.head + 1, self.tabs.len(), self.url_str);
    if self.pending {
      format!(" (pending response) {} ", text)
    } else {text}
  }
}

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
