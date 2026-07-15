// src/tab.rs

use crate::{
  SystemParams,
  TextParams, 
  Rect,
  Frame,
  Cursor, 
  Style, 
  Dialog,
  CursorVec,
  View,
  GetHeight,
  BuildView,
  GemText,
  GemTag,
  Page,
  constants::*,
};
use url::Url;


pub enum TabText {
  Gemini(GemText),
  Gopher(String),
}


impl std::fmt::Display for TabText {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) 
    -> Result<(), std::fmt::Error> 
  {
    match self {
      Self::Gemini(gemtext) => gemtext.fmt(f),
      Self::Gopher(string) => string.fmt(f),
    }
  }
}


impl Page<TabText> {
}


pub struct Tab {
  pub url: Url,
  pub page: Page<TabText>,
} 


impl GetHeight for CursorVec<Tab> {
  fn get_height(&self) -> u16 {
    if let Some(view) = self.get_current() {
      view.page.get_height()
    } else {
      u16::MAX
    }
  }
}


impl View for CursorVec<Tab> {
  fn draw(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
    if let Some(view) = self.get_current() {
      view.page.draw(writer)?;
    }
    Ok(())
  }


  fn resize(&mut self, rect: &Rect) {
    for tab in self.vec.iter_mut() {
      tab.page.resize(rect);
    }
  }
}


pub enum Focus {
  Flash, Dialog, Tabs,
}


pub struct AppView {
  pub frame: Frame,
  pub flash: Option<Page<String>>,
  pub dlg: Option<Dialog>,
  pub tabs: CursorVec<Tab>,
}


impl AppView {
  pub fn get_view_list(&mut self) -> Vec<ViewType> {
    let mut vec: Vec<ViewType> = vec![];
    if let Some(flash) = &mut self.flash {
      vec.push(ViewType::Flash(flash));
    }
    if let Some(dlg) = &mut self.dlg {
      vec.push(ViewType::Dialog(dlg));
    }
    if let Some(tab) = self.tabs.get_current_mut() {
      vec.push(ViewType::Tab(&mut tab.page));
    }
    vec
  }
}


pub enum ViewType<'a> {
  Flash(&'a mut Page<String>),
  Dialog(&'a mut Dialog),
  Tab(&'a mut Page<TabText>),
}


impl<'a> GetHeight for ViewType<'a> {
  fn get_height(&self) -> u16 {
    match self {
      Self::Flash(flash) => {
        flash.get_height()
      }
      Self::Dialog(dialog) => {
        dialog.get_height()
      }
      Self::Tab(page) => {
        page.get_height()
      }
    }
  }
}


impl<'a> View for ViewType<'a> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Flash(flash) => {
        flash.draw(w)?;
      }
      Self::Dialog(dialog) => {
        dialog.draw(w)?;
      }
      Self::Tab(page) => {
        page.draw(w)?;
      }
    }
    Ok(())
  }


  fn resize(&mut self, rect: &Rect) {
    match self {
      Self::Flash(flash) => {
        flash.resize(rect)
      }
      Self::Dialog(dialog) => {
        dialog.resize(rect)
      }
      Self::Tab(page) => {
        page.resize(rect)
      }
    }
  }
}


impl CursorVec<Tab> {
  pub fn get_banner_text(&self) -> String {
    match self.get_current().map(|tab| tab.url.to_string()) {
      None => format!("Empty"),
      Some(s) => format!(
        "{}/{} - {s}", self.cursor.head + 1, self.vec.len()
      ),
    }
  }
}


//impl CursorVec<Tab> {
//  pub fn add_gem_tab(
//    &mut self, 
//    params: &SystemParams,
//    layout: &mut Layout,
//    url:    &Url, 
//    source: Vec<GemText>, 
//  ) {
//    let params = PageViewParams::from(
//      PageParams::init()
//        .with_styled_text(
//          &source, 
//          |g| params.style.get_style_from_gem_text(g),
//        )
//        .with_style(&params.style.general)
//      )
//      .with_draw_point(true);
//    let (tags, text): (Vec<GemTag>, Vec<String>) = source
//      .into_iter()
//      .map(|gemtext| (gemtext.tag, gemtext.string))
//      .unzip();
////  if let Some(insert_command) = self.insert_unique_with(
////    |tab| tab.get_url() == url, 
////    Tab::Gem(TaggedTab::new(url, tags)),
////  ) {
////    layout.apply_insert(TAB, insert_command, params);
////  }
//  }


//pub fn push_gem_style(
//  &mut self, 
//  layout: &mut Layout,
//  style: impl Into<Style>,
//  func: impl Fn(&GemTag) -> TextParams,
//) {
//  let style: Style = style.into();
//  if let Some(views) = layout.map.get_mut(&TAB) {
//  //for (tab, view) in self.vec.iter_mut().zip(views.iter_mut()) {
//  //  if let Tab::Gem(tab) = tab {
//  //    let styles = tab.tags.iter().map(|t| func(t)).collect();
//  //    view.view_params.page_params.set_text_styles(styles);
//  //    view.view_params.page_params.set_style(style);
//  //  }
//  //}
//    layout.push_rebuild();
//  }
//}
