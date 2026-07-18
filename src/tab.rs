// src/tab.rs

use crate::{
  Rect,
  Draw,
  CursorVec,
  Resize,
  GetMaxHeight,
  GemText,
  Page,
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


pub struct Tab {
  pub url: Url,
  pub page: Page<TabText>,
} 


impl GetMaxHeight for CursorVec<Tab> {
  fn get_max_height(&self) -> u16 {
    if let Some(view) = self.get() {
      view.page.get_max_height()
    } else {
      u16::MAX
    }
  }
}


impl Draw for CursorVec<Tab> {
  fn draw(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
    if let Some(view) = self.get() {
      view.page.draw(writer)?;
    }
    Ok(())
  }
}


impl Resize for CursorVec<Tab> {
  fn resize(&mut self, rect: &Rect) {
    for tab in self.data.iter_mut() {
      tab.page.resize(rect);
    }
  }
}


impl CursorVec<Tab> {
  pub fn get_banner_text(&self) -> String {
    match self.get().map(|tab| tab.url.to_string()) {
      None => format!("Empty"),
      Some(s) => format!(
        "{}/{} - {s}", self.cursor.head + 1, self.data.len()
      ),
    }
  }
}
