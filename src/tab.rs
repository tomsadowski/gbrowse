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

  pub fn get_source(&self) ->  Option<&T> {
    self.source.get(
      self.textbox.content.get_source_index()
    )
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
      Tab::Gem(   UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => Some(url),
      _                             => None,
    }
  }

  pub fn heading(&self) -> Option<&str> {
    if let Tab::Text(heading, _) = self {
      Some(heading) 
    } else {None}
  }

  pub fn gem_tab(&self) ->  Option<&UrlTab<GemText>> {
    if let Tab::Gem(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn gopher_tab(&self) ->  Option<&UrlTab<String>> {
    if let Tab::Gopher(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn text_tab(&self) ->  Option<(&str, &TextBox)> {
    if let Tab::Text(heading, textbox) = self {
      Some((heading, textbox))
    } else {None}
  }

  pub fn gem_source(&self) ->  Option<&Vec<GemText>> {
    self.gem_tab().map(|gem_tab| &gem_tab.source)
  }

  pub fn current_gem_source(&self) -> Option<&GemText> {
    self.gem_tab().and_then(|gem_tab| gem_tab.get_source())
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
    self.tabs.iter_mut().map(|tab|
      tab.textbox_mut().style = self.style
    );
    self
  }

  pub fn push_style<T>(&mut self, style: T)
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    self.tabs.iter_mut().map(|tab|
      tab.textbox_mut().style = self.style
    );
  }

  pub fn push_gem_style<F>(&mut self, func: F)
  where F: Fn(&GemText) -> StyledText,
  {
    self.tabs.iter_mut().map(|tab| 
      if let Tab::Gem(gem_tab) = tab {
        let source = &gem_tab.source;
        gem_tab.textbox.restyle(source, |gem| func(gem));
        gem_tab.textbox.style = self.style;
      }
    );
  }

  pub fn resize<V: ViewPort + Copy>(&mut self, view: V) {
    self.view = view.view_port();
    self.tabs.iter_mut().map(|tab|
      tab.textbox_mut().resize(self.view)
    );
  }

  pub fn reset_state(&mut self) {
    self.textbox_mut().map(|textbox|
      textbox.reset_state()
    );
  }

  pub fn textbox_mut(&mut self) -> Option<&mut TextBox> {
    self.get_mut().map(|tab| tab.textbox_mut())
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
    match self.get()
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
    if let Some(tab) = self.get() {
      tab.textbox().write(writer)?;
      tab.textbox().cursor.write(writer)?;
    } else {
      TextBox::from(self.view).empty(writer)?;
    }
    Ok(())
  }
}
