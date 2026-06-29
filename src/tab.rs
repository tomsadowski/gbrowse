// src/tab.rs

use crate::{
  TextStyle, 
  Cursor, 
  Style, 
  PageParams, 
  Page,
  GemText,
  GemTag,
  Layout,
};
use std::{
  collections::HashMap,
  rc::Rc,
};
use url::Url;


#[derive(Default)]
pub struct TabManager {
  pub cursor: Cursor,
  pub tabs:   Vec<Tab>,
} 
impl TabManager {
  pub fn with_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.push_style(style);
    self
  }

  pub fn push_style<T>(&mut self, style: T)
  where T: Into<Style> + Copy
  {
    for tab in self.tabs.iter_mut() {
      tab.get_page_params_mut().style = style.into();
    }
  }

  pub fn push_gem_style<F>(&mut self, func: F)
  where F: Fn(&GemTag) -> TextStyle,
  {
    for tab in self.tabs.iter_mut() {
      if let Tab::Gem(gem_tab) = tab {
        let styles = gem_tab.tags.iter().map(|t| func(t)).collect();
        gem_tab.params.set_text_styles(styles);
      }
    }
  }

  pub fn get_url(&self) -> Option<&url::Url> {
    self.tabs
      .get(*self.cursor)
      .and_then(|tab| tab.get_url())
  }

  pub fn get_gem_tag(&self, page: &Page) -> Option<&GemTag> {
    self.tabs
      .get(*self.cursor)
      .and_then(|tab| tab.get_gem_tag(page))
  }

  pub fn use_page_params_mut<F, T>(&mut self, func: F) -> Option<T>
  where F: Fn(&mut PageParams) -> T
  {
    self.tabs
      .get_mut(*self.cursor)
      .map(|tab| tab.get_page_params_mut())
      .map(|textbox| func(textbox))
  }

  pub fn add_gem_tab<F>(
    &mut self, 
    url:            &url::Url, 
    source:         Vec<GemText>, 
    get_text_style: F
  ) -> Rc<PageParams>
  where F: Fn(&GemText) -> TextStyle,
  {
    let params = PageParams::init().with_text(&source, get_text_style);
    let (tags, text): (Vec<GemTag>, Vec<String>) = source
      .into_iter()
      .map(|gemtext| (gemtext.tag, gemtext.text))
      .unzip();
    let new_tab = Tab::Gem(UrlTab::new(url, tags, params));
    self.cursor.insert_or_move(
      &mut self.tabs, 
      |tab| tab.get_url() == Some(url), 
      new_tab
    );
    self.tabs.get(*self.cursor).map(|t| t.get_page_params())
      .expect("could not retrieve that which we just placed")
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
}

pub enum Tab {
  Text  (String, Rc<PageParams>),
  Gem   (UrlTab<GemTag>),
  Gopher(UrlTab<String>),
}

impl Default for Tab {
  fn default() -> Self {
    Self::Text("".into(), Rc::new(PageParams::default()))
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

  pub fn get_page_params(&self) -> Rc<PageParams> {
    match self {
      Tab::Text(_, textbox) |
      Tab::Gem(   UrlTab {params: textbox, ..}) | 
      Tab::Gopher(UrlTab {params: textbox, ..}) => textbox.clone(),
    }
  }

  pub fn get_page_params_mut(&mut self) -> &mut PageParams {
    match self {
      Tab::Text(_, params) |
      Tab::Gem(   UrlTab {params, ..}) | 
      Tab::Gopher(UrlTab {params, ..}) => params,
    }
  }
}

pub struct UrlTab<T> {
  pub url:    Url,
  pub tags:   Vec<T>,
  pub params: Rc<PageParams>,
} 

impl<T> UrlTab<T> {
  pub fn new(url: &Url, tags: Vec<T>, page_params: PageParams) -> Self {
    Self {
      url: url.clone(),
      params: Rc::new(page_params),
      tags,
    }
  }

  pub fn moop(&mut self) {
    self.params.style = Style::default();
  }

  pub fn get_current_tag(&self, page: &Page) -> Option<&T> {
    self.tags.get(page.get_current_index())
  }
}
