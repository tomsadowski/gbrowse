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
  pub fn get_heading(&self) -> &str {
    match self {
      Tab::Gem(   UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => url.as_str(),
      Tab::Text(heading, _)         => heading,
    }
  }

  pub fn get_url(&self) -> Option<&url::Url> {
    match self {
      Tab::Gem(   UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => Some(url),
      _                             => None,
    }
  }

  pub fn get_gem_tab(&self) ->  Option<&UrlTab<GemText>> {
    if let Tab::Gem(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn get_gopher_tab(&self) ->  Option<&UrlTab<String>> {
    if let Tab::Gopher(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn get_text_tab(&self) ->  Option<(&str, &TextBox)> {
    if let Tab::Text(heading, textbox) = self {
      Some((heading, textbox))
    } else {None}
  }

  pub fn get_gem_source(&self) ->  Option<&Vec<GemText>> {
    self.get_gem_tab().map(|gem_tab| &gem_tab.source)
  }

  pub fn get_current_gem_source(&self) -> Option<&GemText> {
    self.get_gem_tab().and_then(|gem_tab| gem_tab.get_source())
  }

  pub fn get_textbox(&self) -> &TextBox {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }

  pub fn get_textbox_mut(&mut self) -> &mut TextBox {
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
  fn get_units(&self) -> &Vec<Tab> {
    &self.tabs
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
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
      view:  view.get_view_port(),
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
      tab.get_textbox_mut().style = self.style
    );
    self
  }

  pub fn push_style<T>(&mut self, style: T)
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    self.tabs.iter_mut().map(|tab|
      tab.get_textbox_mut().style = self.style
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
    self.view = view.get_view_port();
    self.tabs.iter_mut().map(|tab|
      tab.get_textbox_mut().resize(self.view)
    );
  }

  pub fn reset_state(&mut self) {
    self.textbox_mut().map(|textbox|
      textbox.reset_state()
    );
  }

  pub fn textbox_mut(&mut self) -> Option<&mut TextBox> {
    self.get_current_mut().map(|tab| tab.get_textbox_mut())
  }

  pub fn add_gem_tab<F>(
    &mut self, 
    url:    &url::Url, 
    source: Vec<GemText>, 
    func:   F
  ) 
  where F: Fn(&GemText) -> StyledText,
  {
    let new_tab = Tab::Gem(UrlTab::new(url, self.view, source, func));
    self.insert_or_move(|tab| tab.get_url() == Some(url), new_tab);
    self.reset_state();
  }

  pub fn get_banner_text(&self) -> String {
    match self.get_current()
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
    if let Some(tab) = self.get_current() {
      tab.get_textbox().write(writer)?;
      tab.get_textbox().cursor.write(writer)?;
    } else {
      TextBox::from(self.view).empty(writer)?;
    }
    Ok(())
  }
}
