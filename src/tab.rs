// src/tab.rs

use crate::{
  SystemParams,
  TextParams, 
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


pub struct TaggedTab<T> {
  pub url:  Url,
  pub tags: Vec<T>,
} 


impl<T> TaggedTab<T> {
  pub fn new(url: &Url, tags: Vec<T>) -> Self {
    Self { url: url.clone(), tags }
  }


  pub fn get_current_tag(&self, page: &Page) -> Option<&T> {
    self.tags.get(page.get_index())
  }
}


pub enum Tab {
  Gem(TaggedTab<GemTag>),
  Gopher(TaggedTab<String>),
}


impl Tab {
  pub fn get_url(&self) -> &Url {
    match self {
      Tab::Gem(TaggedTab {url, ..}) | 
      Tab::Gopher(TaggedTab {url, ..}) => url
    }
  }


  pub fn get_gem_tab(&self) ->  Option<&TaggedTab<GemTag>> {
    if let Tab::Gem(tab) = self {
      Some(tab)
    } else {None}
  }


  pub fn get_gopher_tab(&self) ->  Option<&TaggedTab<String>> {
    if let Tab::Gopher(tab) = self {
      Some(tab)
    } else {None}
  }
}


impl CursorVec<Tab> {
  pub fn add_gem_tab(
    &mut self, 
    params: &SystemParams,
    layout: &mut Layout,
    url:    &Url, 
    source: Vec<GemText>, 
  ) {
    let params = PageViewParams::from(
      PageParams::init()
        .with_styled_text(
          &source, 
          |g| params.style.get_style_from_gem_text(g),
        )
        .with_style(&params.style.general)
      )
      .with_draw_point(true);
    let (tags, text): (Vec<GemTag>, Vec<String>) = source
      .into_iter()
      .map(|gemtext| (gemtext.tag, gemtext.text))
      .unzip();
    if let Some(insert_command) = self.insert_unique_with(
      |tab| tab.get_url() == url, 
      Tab::Gem(TaggedTab::new(url, tags)),
    ) {
      layout.apply_insert(TAB, insert_command, params);
    }
  }


  pub fn push_gem_style(
    &mut self, 
    layout: &mut Layout,
    style:  impl Into<Style> + Copy,
    func:   impl Fn(&GemTag) -> TextParams,
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
    match self.get_current().map(|tab| tab.get_url().to_string()) {
      None    => format!("Empty"),
      Some(s) => format!("{}/{} - {s}", *self.cursor + 1, self.vec.len()),
    }
  }
}
