// src/tab.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut},
  view::{Rect, ViewPort},
  widget::TextBox,
  gemdoc::GemText,
};


pub struct TabList {
  pub head: usize,
  pub tabs: Vec<Tab>,
} 
impl UnitCursor for TabList {
  type Unit = Tab;
  fn units(&self) -> &Vec<Tab> {
    &self.tabs
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
  fn max_head(&self) -> usize {
    self.tabs.len().saturating_sub(1)
  }
}
impl UnitCursorMut for TabList {
  fn units_mut(&mut self) -> &mut Vec<Tab> {
    &mut self.tabs
  }
}
impl std::ops::Deref for TabList {
  type Target = Tab;
  fn deref(&self) -> &Self::Target {
    &self.tabs[self.head]
  }
}
impl std::ops::DerefMut for TabList {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.tabs[self.head]
  }
}
impl TabList {
  pub fn new(tab: Tab) -> Self {
    Self {tabs: vec![tab], head: 0}
  }

  pub fn banner_text(&self) -> String {
    format!("{}/{} - {}", self.head + 1, self.tabs.len(), self.url)
  }

  // maybe return bool
  pub fn add(&mut self, url: &url::Url) {
    // search for tab with same url_str
    // move head to location of tab with url_str
    if let Some((idx, _)) = self.tabs
      .iter_mut()
      .enumerate()
      .find(|(_, tab)| tab.url == *url)
    {
      self.head = idx;
    } else {
      let new_tab = Tab::init(self.view, url);
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

  pub fn resize<V: ViewPort + Copy>(&mut self, rect: V) {
    for tab in self.tabs.iter_mut() {
      tab.resize(rect);
    }
  }
}

pub struct Tab {
  pub url:     url::Url,
  pub source:  Vec<GemText>,
  pub content: TextBox,
} 
impl std::ops::Deref for Tab {
  type Target = TextBox;
  fn deref(&self) -> &Self::Target {
    &self.content
  }
}
impl std::ops::DerefMut for Tab {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.content
  }
}
impl Tab {
  pub fn init<V: ViewPort>(rect: V, url: &url::Url) -> Self {
    let mut content = TextBox::default();
    content.view    = rect.view_port();
    Self {
      url:    url.clone(),
      source: vec![],
      content, 
    }
  }
}
