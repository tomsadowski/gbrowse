// src/tab.rs

use crate::{
  StyledText, 
  TextLine,
  UnitCursor, 
  UnitCursorMut,
  Style, 
  Rect, 
  ViewPort, 
  TextBox, 
  GemText,
};
use url::Url;
use std::io::Write;


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
    for tab in self.tabs.iter_mut() {
      tab.get_textbox_mut().style = self.style;
    }
    self
  }

  pub fn push_style<T>(&mut self, style: T)
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    for tab in self.tabs.iter_mut() {
      tab.get_textbox_mut().style = self.style;
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
    self.view = view.get_view_port();
    for tab in self.tabs.iter_mut() {
      tab.get_textbox_mut().resize(self.view);
    }
  }

  pub fn reset_state(&mut self) {
    self.use_current_mut(
      |tab| {
        tab.get_textbox_mut().reset_state();
      }
    );
  }

  pub fn get_url(&self) -> Option<&Url> {
    self
      .get_current()
      .and_then(|tab| tab.get_url())
  }

  pub fn use_gem_text<F, T>(&self, func: F) -> Option<T>
  where F: Fn(&GemText) -> T
  {
    self
      .get_current()
      .and_then(|tab| tab.get_gem_text())
      .map(|gem_text| func(gem_text))
  }

  pub fn use_textbox_mut<F, T>(&mut self, func: F) -> Option<T>
  where F: Fn(&mut TextBox<TextLine>) -> T
  {
    self
      .get_current_mut()
      .map(|tab| tab.get_textbox_mut())
      .map(|textbox| func(textbox))
  }

  pub fn add_gem_tab<F>(
    &mut self, 
    url:    &Url, 
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
    match self.use_current(
      |tab| match tab {
        Tab::Gem(   UrlTab {url, ..}) | 
        Tab::Gopher(UrlTab {url, ..}) => url.to_string(),
        Tab::Text(heading, _)         => heading.to_string(),
      }) 
    {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {s}", self.head + 1, self.tabs.len()),
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W, overlay: u16) 
    -> std::io::Result<()> 
  {
    if let Some(tab) = self.get_current() {
      tab.get_textbox().write(writer, overlay)?;
      tab.get_textbox().cursor.write(writer)?;
    } else {
      let tb: TextBox<TextLine> = self.view.into();
      tb.empty(writer)?;
    }
    Ok(())
  }
}

pub enum Tab {
  Text(String, TextBox<TextLine>),
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

  pub fn get_url(&self) -> Option<&Url> {
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

  pub fn get_text_tab(&self) ->  Option<(&str, &TextBox<TextLine>)> {
    if let Tab::Text(heading, textbox) = self {
      Some((heading, textbox))
    } else {None}
  }

  pub fn get_gem_source(&self) ->  Option<&Vec<GemText>> {
    self.get_gem_tab().map(|gem_tab| &gem_tab.source)
  }

  pub fn get_gem_text(&self) -> Option<&GemText> {
    self
      .get_gem_tab()
      .and_then(|gem_tab| gem_tab.get_current_source())
  }

  pub fn get_textbox(&self) -> &TextBox<TextLine> {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }

  pub fn get_textbox_mut(&mut self) -> &mut TextBox<TextLine> {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }
}

pub struct UrlTab<T> {
  pub url:     Url,
  pub source:  Vec<T>,
  pub textbox: TextBox<TextLine>,
} 

impl<T> UrlTab<T> {
  pub fn new<V, F>(
    url:            &Url, 
    view:           V, 
    source:         Vec<T>, 
    to_styled_text: F
  ) -> Self
  where 
    V: ViewPort, 
    F: Fn(&T) -> StyledText
  {
    Self {
      url:     url.clone(),
      textbox: TextBox::from(view).reference(&source, to_styled_text),
      source,
    }
  }

  pub fn get_current_source(&self) -> Option<&T> {
    self.source.get(
      self.textbox.get_current_reference_index()
    )
  }
}
