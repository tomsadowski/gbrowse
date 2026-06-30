// src/tab.rs

use crate::{
  TextStyle, 
  Cursor, 
  Style, 
  PageParams, 
  PageView,
  CursorVec,
  Page,
  GemText,
  GemTag,
  Layout,
  PageViewParams,
  constants::*,
};
use url::Url;


impl CursorVec<Tab> {
  pub fn push_gem_style<F, S>(
    &mut self, 
    layout: &mut Layout,
    style:  S,
    func:   F,
  ) 
  where F: Fn(&GemTag) -> TextStyle,
        S: Into<Style> + Copy,
  {
    if let Some(views) = layout.map.get_mut(&TAB) {
      for (tab, view) in self.vec.iter_mut().zip(views.iter_mut()) {
        if let Tab::Gem(tab) = tab {
          let styles = tab.tags.iter().map(|t| func(t)).collect();
          view.view_params.page_params.set_text_styles(styles);
          view.view_params.page_params.set_style(style);
        }
      }
      layout.push_rebuild();
    }
  }

  pub fn add_gem_tab<F, S>(
    &mut self, 
    layout:         &mut Layout,
    url:            &url::Url, 
    source:         Vec<GemText>, 
    style:          S,
    get_text_style: F
  ) 
  where F: Fn(&GemText) -> TextStyle,
        S: Into<Style> + Copy,
  {
    let params = PageParams::init().with_styled_text(&source, get_text_style);
    let (tags, text): (Vec<GemTag>, Vec<String>) = source
      .into_iter()
      .map(|gemtext| (gemtext.tag, gemtext.text))
      .unzip();
    let new_tab = Tab::Gem(UrlTab::new(url, tags));
    if let Some(cursor_insert) = self.cursor.insert_unique_with(
      &mut self.vec, 
      |tab| tab.get_url() == Some(url), 
      new_tab
    ) {
      layout.apply_insert_command(
        TAB, cursor_insert, PageViewParams::from(params)
      );
    }
  }
  pub fn get_url(&self) -> Option<&url::Url> {
    self.vec
      .get(*self.cursor)
      .and_then(|tab| tab.get_url())
  }
  pub fn get_gem_tag(&self, page: &Page) -> Option<&GemTag> {
    self.vec
      .get(*self.cursor)
      .and_then(|tab| tab.get_gem_tag(page))
  }
  pub fn get_banner_text(&self) -> String {
    match self.vec.get(*self.cursor).map(
      |tab| match tab {
        Tab::Gem   (UrlTab {url, ..}) | 
        Tab::Gopher(UrlTab {url, ..}) => url.to_string(),
        Tab::Text  (heading, _)       => heading.to_string(),
      }
    ) {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {s}", *self.cursor + 1, self.vec.len()),
    }
  }
}

pub enum Tab {
  Text  (String, PageParams),
  Gem   (UrlTab<GemTag>),
  Gopher(UrlTab<String>),
}
impl Default for Tab {
  fn default() -> Self {
    Self::Text("".into(), PageParams::default())
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
  pub fn get_text_tab(&self) ->  Option<(&str, &PageParams)> {
    if let Tab::Text(heading, params) = self {
      Some((heading, params))
    } else {None}
  }
  pub fn get_gem_tag(&self, page: &Page) -> Option<&GemTag> {
    self
      .get_gem_tab()
      .and_then(|gem_tab| gem_tab.get_current_tag(page))
  }
}

pub struct UrlTab<T> {
  pub url:  Url,
  pub tags: Vec<T>,
} 
impl<T> UrlTab<T> {
  pub fn new(url: &Url, tags: Vec<T>) -> Self {
    Self { url: url.clone(), tags }
  }
  pub fn get_current_tag(&self, page: &Page) -> Option<&T> {
    self.tags.get(page.get_index())
  }
}
