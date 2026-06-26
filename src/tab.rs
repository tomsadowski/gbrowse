// src/tab.rs

use crate::{
  TextStyle, 
  Cursor, 
  Style, 
  Rect, 
  GetRect, 
  TextCursor, 
  GemText,
  GemTag,
};


pub struct TabManager {
  pub view:   Rect,
  pub style:  Style,
  pub cursor: Cursor,
  pub tabs:   Vec<Tab>,
} 

impl<V: GetRect> From<V> for TabManager {
  fn from(view: V) -> Self {
    Self {
      view:   view.get_rect(),
      style:  Style::default(),
      cursor: Cursor::default(),
      tabs:   vec![],
    }
  }
}

impl TabManager {
  pub fn with_style<T: Into<Style> + Copy>(mut self, style: T) -> Self {
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
  where F: Fn(&GemTag) -> TextStyle,
  {
    for tab in self.tabs.iter_mut() {
      if let Tab::Gem(gem_tab) = tab {
        let styles = gem_tab.tags.iter().map(|t| func(t)).collect();
        gem_tab.textbox.set_styles(styles);
        gem_tab.textbox.style = self.style;
      }
    }
  }

  pub fn resize<V: GetRect + Copy>(&mut self, view: V) {
    self.view = view.get_rect();
    for tab in self.tabs.iter_mut() {
      tab.get_textbox_mut().resize(self.view);
    }
  }

  pub fn reset_state(&mut self) {
    self.tabs.get_mut(*self.cursor).map(
      |tab| tab.get_textbox_mut().reset_state()
    );
  }

  pub fn get_url(&self) -> Option<&url::Url> {
    self.tabs
      .get(*self.cursor)
      .and_then(|tab| tab.get_url())
  }

  pub fn get_gem_tag(&self) -> Option<&GemTag> {
    self.tabs
      .get(*self.cursor)
      .and_then(|tab| tab.get_gem_tag())
  }

  pub fn use_textbox_mut<F, T>(&mut self, func: F) -> Option<T>
  where F: Fn(&mut TextCursor) -> T
  {
    self.tabs
      .get_mut(*self.cursor)
      .map(|tab| tab.get_textbox_mut())
      .map(|textbox| func(textbox))
  }

  pub fn add_gem_tab<F>(
    &mut self, 
    url:            &url::Url, 
    source:         Vec<GemText>, 
    get_text_style: F
  ) 
  where F: Fn(&GemTag) -> TextStyle,
  {
    let (tags, text): (Vec<GemTag>, Vec<String>) = source
      .into_iter()
      .map(|gemtext| (gemtext.tag, gemtext.text))
      .unzip();
    let styles  = tags.iter().map(|tag| get_text_style(tag)).collect();
    let new_tab = Tab::Gem(UrlTab::new(self.view, url, tags, text, styles));
    self.cursor.insert_or_move(
      &mut self.tabs, 
      |tab| tab.get_url() == Some(url), 
      new_tab
    );
    self.reset_state();
  }

  pub fn get_banner_text(&self) -> String {
    match self.tabs.get(*self.cursor).map(
      |tab| match tab {
        Tab::Gem   (UrlTab {url, ..}) | 
        Tab::Gopher(UrlTab {url, ..}) => url.to_string(),
        Tab::Text  (heading, _)       => heading.to_string(),
      }
    ) {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {s}", *self.cursor + 1, self.tabs.len()),
    }
  }

  pub fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    if let Some(tab) = self.tabs.get(*self.cursor) {
      tab.get_textbox().draw_cursor(w)?;
      tab.get_textbox().cursor.draw(w)?;
    } 
    Ok(())
  }
}

pub enum Tab {
  Text  (String, TextCursor),
  Gem   (UrlTab<GemTag>),
  Gopher(UrlTab<String>),
}

impl Default for Tab {
  fn default() -> Self {
    Self::Text("".into(), TextCursor::default())
  }
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

  pub fn get_gem_tab(&self) ->  Option<&UrlTab<GemTag>> {
    if let Tab::Gem(tab) = self {Some(tab)} else {None}
  }

  pub fn get_gopher_tab(&self) ->  Option<&UrlTab<String>> {
    if let Tab::Gopher(tab) = self {Some(tab)} else {None}
  }

  pub fn get_text_tab(&self) ->  Option<(&str, &TextCursor)> {
    if let Tab::Text(heading, textbox) = self {
      Some((heading, textbox))
    } else {None}
  }

  pub fn get_gem_tag(&self) -> Option<&GemTag> {
    self
      .get_gem_tab()
      .and_then(|gem_tab| gem_tab.get_current_tag())
  }

  pub fn get_textbox(&self) -> &TextCursor {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }

  pub fn get_textbox_mut(&mut self) -> &mut TextCursor {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {textbox, ..}) | 
      Tab::Gopher(UrlTab {textbox, ..}) => textbox,
    }
  }
}

pub struct UrlTab<T> {
  pub url:     url::Url,
  pub tags:    Vec<T>,
  pub textbox: TextCursor,
} 

impl<T> UrlTab<T> {
  pub fn new<V: GetRect>(
    view:   V, 
    url:    &url::Url, 
    tags:   Vec<T>, 
    text:   Vec<String>,
    styles: Vec<TextStyle>,
  ) -> Self {
    Self {
      textbox: TextCursor::from(view.get_rect()).text(text, styles),
      url:     url.clone(),
      tags,
    }
  }

  pub fn get_current_tag(&self) -> Option<&T> {
    self.tags.get(
      self.textbox.get_current_index()
    )
  }
}
