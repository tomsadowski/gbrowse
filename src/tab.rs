// src/tab.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut},
  view::{Rect, ViewPort},
  widget::TextBox,
  gemdoc::GemText,
  text::StyledText,
};
use std::io::Write;


pub struct TabManager {
  pub view: Rect,
  pub head: usize,
  pub tabs: Vec<Tab>,
} 
impl<V: ViewPort> From<V> for TabManager {
  fn from(view: V) -> Self {
    Self {
      view: view.view_port(),
      head: 0,
      tabs: vec![],
    }
  }
}
impl UnitCursor for TabManager {
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
impl UnitCursorMut for TabManager {
  fn units_mut(&mut self) -> &mut Vec<Tab> {
    &mut self.tabs
  }
}
impl TabManager {
  pub fn banner_text(&self) -> String {
    match self.current_checked() {
      None => 
        format!("0/0 - No URL"),
      Some(tab) => 
        format!("{}/{} - {}", self.head + 1, self.tabs.len(), tab.url),
    }
  }

  pub fn reset_state(&mut self) {
    if let Some(tab) = self.current_mut_checked() {
      tab.content.reset_state();
    }
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
      if self.tabs.len() == 0 {
        self.tabs.push(new_tab);
      } else if self.head + 1 == self.tabs.len() {
        self.tabs.push(new_tab);
        self.head += 1;
      }
      else {
        self.head += 1;
        self.tabs.insert(self.head, new_tab);
      }
    }
    self.current_mut().content.reset_state();
  }

  pub fn delete(&mut self) -> usize {
    if self.tabs.len() > 0 {
      self.tabs.remove(self.head);
      if self.tabs.len() > 0 {
        self.wrapping_backward(1);
      }
    }
    self.tabs.len()
  }

  pub fn resize<V: ViewPort + Copy>(&mut self, rect: V) {
    for tab in self.tabs.iter_mut() {
      tab.content.resize(rect);
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    match self.current_checked() {
      None => {
        TextBox::from(self.view).empty(writer)?;
      }
      Some(tab) => {
        tab.content.write(writer)?;
        tab.content.cursor.write(writer)?;
      }
    }
    Ok(())
  }
}

pub struct Tab {
  pub url:     url::Url,
  pub source:  Vec<GemText>,
  pub content: TextBox,
} 
impl Tab {
  pub fn init<V: ViewPort>(view: V, url: &url::Url) -> Self {
    Self {
      url:     url.clone(),
      source:  vec![],
      content: TextBox::from(view), 
    }
  }
  pub fn set_source<F>(
    &mut self, 
    source: Vec<GemText>, 
    func:   F,
  ) 
  where F: Fn(&GemText) -> StyledText,
  {
    self.source = source;
    self.content.set_input(&self.source, func);
  }
}
