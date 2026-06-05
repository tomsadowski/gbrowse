// src/tab.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut},
  view::{Rect, ViewPort},
  widget::TextBox,
  gemdoc::GemText,
  text::StyledText,
};
use std::io::Write;


pub struct Tab<T> {
  pub url:     url::Url,
  pub source:  Vec<T>,
  pub textbox: TextBox,
} 
impl<T> Tab<T> {
  pub fn new<V, F>(url: &url::Url, view: V, source: Vec<T>, func: F) -> Self
  where V: ViewPort, F: Fn(&T) -> StyledText
  {
    Self {
      url:     url.clone(),
      textbox: TextBox::from(view).with_input(&source, func),
      source,
    }
  }
}

pub enum SourceType<'a> {
  Text(String),
  Gem(&'a GemText),
  Gopher(&'a str),
}

pub enum TabType {
  Text(String, TextBox),
  Gem(Tab<GemText>),
  Gopher(Tab<String>),
}
impl TabType {
  pub fn url(&self) -> Option<&url::Url> {
    match self {
      TabType::Text(_, _) => None,
      TabType::Gem(   Tab {url: url, ..}) | 
      TabType::Gopher(Tab {url: url, ..}) => 
        Some(url),
    }
  }

  pub fn current_source(&self) ->  SourceType {
    match self { 
      TabType::Text(_, textbox) => 
        SourceType::Text(textbox.content.get_source()),
      TabType::Gem(tab) => 
        SourceType::Gem(&tab.source[tab.textbox.get_source_idx()]),
      TabType::Gopher(tab) => 
        SourceType::Gopher(&tab.source[tab.textbox.get_source_idx()]),
    }
  }

  pub fn heading(&self) -> Option<&str> {
    match self {
      TabType::Text(heading, _) => Some(heading),
      _ => None,
    }
  }

  pub fn textbox(&self) -> &TextBox {
    match self {
      TabType::Text(_, textbox) |
      TabType::Gem(   Tab {textbox, ..}) | 
      TabType::Gopher(Tab {textbox, ..}) => 
        textbox,
    }
  }

  pub fn textbox_mut(&mut self) -> &mut TextBox {
    match self {
      TabType::Text(_, textbox) |
      TabType::Gem(   Tab {textbox, ..}) | 
      TabType::Gopher(Tab {textbox, ..}) => 
        textbox,
    }
  }

  pub fn reset_state(&mut self) {
    self.textbox_mut().reset_state()
  }
}

pub struct TabManager {
  pub view: Rect,
  pub head: usize,
  pub tabs: Vec<TabType>,
} 
impl<V: ViewPort> From<V> for TabManager {
  fn from(view: V) -> Self {
    Self {
      view: view.view_port(),
      head: 0,
      tabs: vec![TabType::Text("".into(), TextBox::from(view))],
    }
  }
}
impl UnitCursor for TabManager {
  type Unit = TabType;
  fn units(&self) -> &Vec<TabType> {
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
  fn units_mut(&mut self) -> &mut Vec<TabType> {
    &mut self.tabs
  }
}
impl TabManager {
  pub fn banner_text(&self) -> String {
    match self.current() {
      TabType::Text(heading, _) => 
        format!("{}/{} - {}", self.head + 1, self.tabs.len(), heading),
      TabType::Gem(   Tab {url: url, ..}) | 
      TabType::Gopher(Tab {url: url, ..}) => 
        format!("{}/{} - {}", self.head + 1, self.tabs.len(), url),
    }
  }

  pub fn current_url(&self) -> Option<&url::Url> {
    self.current().url()
  }

  pub fn current_textbox(&self) -> &TextBox {
    self.current().textbox()
  }

  pub fn current_textbox_mut(&mut self) -> &mut TextBox {
    self.current_mut().textbox_mut()
  }

  pub fn reset_state(&mut self) {
    self.current_mut().textbox_mut().reset_state()
  }

  // maybe return bool
  pub fn add_gem<F>(&mut self, url: &url::Url, source: Vec<GemText>, func: F) 
  where F: Fn(&GemText) -> StyledText,
  {
    // search for tab with same url_str
    // move head to location of tab with url_str
    if let Some((idx, _)) = self.tabs
      .iter_mut()
      .enumerate()
      .find(|(_, tab)| tab.url() == Some(url))
    {
      self.head = idx;
    } else {
      let new_tab = Tab::new(url, self.view, source, func);
      if self.tabs.len() == 0 {
        self.tabs.push(TabType::Gem(new_tab));
      } else if self.head + 1 == self.tabs.len() {
        self.tabs.push(TabType::Gem(new_tab));
        self.head += 1;
      }
      else {
        self.head += 1;
        self.tabs.insert(self.head, TabType::Gem(new_tab));
      }
    }
    self.current_mut().textbox_mut().reset_state();
  }

  pub fn delete(&mut self) -> usize {
    if self.tabs.len() > 1 {
      self.tabs.remove(self.head);
      self.wrapping_backward(1);
    }
    self.tabs.len()
  }

  pub fn resize<V: ViewPort + Copy>(&mut self, view: V) {
    self.view = view.view_port();
    for tab in self.tabs.iter_mut() {
      tab.textbox_mut().resize(self.view);
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    self.current().textbox().write(writer)?;
    self.current().textbox().cursor.write(writer)?;
    Ok(())
  }
}
