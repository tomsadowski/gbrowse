// src/tab.rs

use crate::{
  TextParams, 
  Rect,
  Draw,
  CursorVec,
  Resize,
  GetHeight,
  GemText,
  Page,
  PageParams,
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
  pub fn gem_styles(
    &mut self, 
    get_style: impl Fn(&GemText) -> TextParams,
  ) {
    self.styles = self.source
      .iter()
      .zip(self.styles.iter())
      .map(|(tabtext, style)| 
        if let TabText::Gemini(gemtext) = tabtext {
          get_style(&gemtext)
        } else {
          style.clone()
        }
      )
      .collect();
    self.rebuild(&Rect::from(&self.point_view));
  }
}


impl PageParams<TabText> {
  pub fn gem_styles(
    mut self, 
    text: Vec<GemText>, 
    get_style: impl Fn(&GemText) -> TextParams,
  ) -> Self 
  {
    self.styles = text.iter().map(|t| get_style(t)).collect();
    self.source = text.into_iter().map(TabText::Gemini).collect();
    self
  }
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


impl Draw for CursorVec<Tab> {
  fn draw(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
    if let Some(view) = self.get_current() {
      view.page.draw(writer)?;
    }
    Ok(())
  }
}


impl Resize for CursorVec<Tab> {
  fn resize(&mut self, rect: &Rect) {
    for tab in self.vec.iter_mut() {
      tab.page.resize(rect);
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
