// src/tab.rs

use crate::{
  text::{Linear}, 
  widget::{Rect, TextBox},
  protocol::{GemDoc},
};
use std::{
  ops::{Deref, DerefMut},
};


pub struct TabList {
  pub head:      usize,
  pub tabs:      Vec<Tab>,
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
  pub fn banner_text(&self) -> String {
    format!(" {}/{} - {} ", self.head + 1, self.tabs.len(), self.url_str)
  }
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
    Self {tabs: vec![tab], head: 0}
  }
  pub fn resize(&mut self, rect: &Rect) {
    for tab in self.tabs.iter_mut() {
      tab.resize(rect);
    }
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
