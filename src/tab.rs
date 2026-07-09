// src/tab.rs

use crate::{
  User,
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

pub enum Tab {
  Text(String, PageParams),
  Gem(UrlTab<GemTag>),
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
      Tab::Gem(UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => url.as_str(),
      Tab::Text(heading, _)         => heading,
    }
  }

  pub fn get_url(&self) -> Option<&url::Url> {
    match self {
      Tab::Gem(UrlTab {url, ..}) | 
      Tab::Gopher(UrlTab {url, ..}) => Some(url),
      _                             => None,
    }
  }

  pub fn get_gem_tab(&self) ->  Option<&UrlTab<GemTag>> {
    if let Tab::Gem(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn get_gopher_tab(&self) ->  Option<&UrlTab<String>> {
    if let Tab::Gopher(tab) = self {
      Some(tab)
    } else {None}
  }

  pub fn get_text_tab(&self) ->  Option<(&str, &PageParams)> {
    if let Tab::Text(heading, params) = self {
      Some((heading, params))
    } else {None}
  }
}

impl CursorVec<Tab> {
  pub fn add_gem_tab(
    &mut self, 
    params:         &User,
    layout:         &mut Layout,
    url:            &url::Url, 
    source:         Vec<GemText>, 
  ) {
    let params = PageViewParams::from(
      PageParams::init()
        .with_styled_text(
          &source, 
          |g| params.style.get_style_from_gem_text(g),
        )
        .with_style(params.style.general)
      )
      .with_draw_point(true);
    let (tags, text): (Vec<GemTag>, Vec<String>) = source
      .into_iter()
      .map(|gemtext| (gemtext.tag, gemtext.text))
      .unzip();
    if let Some(insert_command) = self.insert_unique_with(
      |tab| tab.get_url() == Some(url), 
      Tab::Gem(UrlTab::new(url, tags)),
    ) {
      layout.apply_insert(TAB, insert_command, params);
    }
  }

  pub fn push_gem_style(
    &mut self, 
    layout: &mut Layout,
    style:  impl Into<Style> + Copy,
    func:   impl Fn(&GemTag) -> TextStyle,
  ) {
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

  pub fn get_banner_text(&self) -> String {
    match self.vec.get(*self.cursor).map(
      |tab| match tab {
        Tab::Gem(UrlTab {url, ..}) | 
        Tab::Gopher(UrlTab {url, ..}) => url.to_string(),
        Tab::Text(heading, _)         => heading.to_string(),
      }
    ) {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {s}", *self.cursor + 1, self.vec.len()),
    }
  }
}
