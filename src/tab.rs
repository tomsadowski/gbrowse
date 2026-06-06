// src/tab.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut},
  view::{Rect, ViewPort},
  widget::TextBox,
  gemdoc::GemText,
  text::StyledText,
  style::Style,
};
use std::io::Write;


pub struct UrlTab<T> {
  pub url:     url::Url,
  pub source:  Vec<T>,
  pub textbox: TextBox,
} 
impl<T> UrlTab<T> {
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

pub enum Tab {
  Text(String, TextBox),
  Gem(UrlTab<GemText>),
  Gopher(UrlTab<String>),
}
impl Tab {
  pub fn url(&self) -> Option<&url::Url> {
    match self {
      Tab::Text(_, _)               => None,
      Tab::Gem(   UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => Some(url),
    }
  }

  pub fn heading(&self) -> Option<&str> {
    match self {
      Tab::Text(heading, _) => Some(heading),
      _                     => None,
    }
  }

  pub fn gem_tab(&self) ->  Option<&UrlTab<GemText>> {
    match self {
      Tab::Gem(tab) => Some(tab),
      _             => None,
    }
  }

  pub fn gopher_tab(&self) ->  Option<&UrlTab<String>> {
    match self {
      Tab::Gopher(tab) => Some(tab),
      _                => None,
    }
  }

  pub fn text_tab(&self) ->  Option<(&str, &TextBox)> {
    match self {
      Tab::Text(heading, textbox) => Some((heading, textbox)),
      _                           => None,
    }
  }

  pub fn gem_source(&self) ->  Option<&Vec<GemText>> {
    self.gem_tab().map(|tab| &tab.source)
  }

  pub fn current_gem_source(&self) ->  Option<&GemText> {
    self
      .gem_tab()
      .map(|tab| &tab.source[tab.textbox.get_source_idx()])
  }

  pub fn textbox(&self) -> &TextBox {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }

  pub fn textbox_mut(&mut self) -> &mut TextBox {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }
}

pub struct TabManager {
  pub view:  Rect,
  pub style: Style,
  pub head:  usize,
  pub tabs:  Vec<Tab>,
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
impl<V: ViewPort> From<V> for TabManager {
  fn from(view: V) -> Self {
    Self {
      view:  view.view_port(),
      style: Style::default(),
      head:  0,
      tabs:  vec![],
    }
  }
}
impl TabManager {
  pub fn with_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    for tab in self.tabs.iter_mut() {
      tab.textbox_mut().style = self.style;
    }
    self
  }

  pub fn push_style<T>(&mut self, style: T)
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    for tab in self.tabs.iter_mut() {
      tab.textbox_mut().style = self.style;
    }
  }

  pub fn push_gem_style<F>(&mut self, func: F)
  where F: Fn(&GemText) -> StyledText,
  {
    for tab in self.tabs.iter_mut() {
      if let Tab::Gem(gem_tab) = tab {
        let source = &gem_tab.source;
        gem_tab.textbox.restyle(source, |gem| func(gem));
        gem_tab.textbox.style = self.style;
      }
    }
  }

  pub fn resize<V: ViewPort + Copy>(&mut self, view: V) {
    self.view = view.view_port();
    for tab in self.tabs.iter_mut() {
      tab.textbox_mut().resize(self.view);
    }
  }

  pub fn reset_state(&mut self) {
    if let Some(tab) = self.current_mut_checked() {
      tab.textbox_mut().reset_state()
    }
  }

  // maybe return bool
  pub fn add_gem<F>(&mut self, url: &url::Url, source: Vec<GemText>, func: F) 
  where F: Fn(&GemText) -> StyledText,
  {
    let new_tab = Tab::Gem(UrlTab::new(url, self.view, source, func));
    self.insert_or_move(|tab| tab.url() == Some(url), new_tab);
    self.reset_state();
  }

  pub fn banner_text(&self) -> String {
    match self.current_checked()
      .map(|tab| match tab {
        Tab::Gem(   UrlTab {url, ..}) | 
        Tab::Gopher(UrlTab {url, ..}) => url.to_string(),
        Tab::Text(heading, _)         => heading.to_string(),
      }) 
    {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {}", self.head + 1, self.tabs.len(), s),
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    if let Some(tab) = self.current_checked() {
      tab.textbox().write(writer)?;
      tab.textbox().cursor.write(writer)?;
    } else {
      TextBox::from(self.view).empty(writer)?;
    }
    Ok(())
  }
}
