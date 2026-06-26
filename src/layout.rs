// src/layout.rs

use crate::{
  Rect, 
  GetRect,
  Frame,
  ScreenCursor,
  TextCursor,
};
use std::{
  collections::HashMap,
  rc::Rc,
};


pub trait GetHeight { 
  fn get_height(&self) -> u16; 
}

impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 { 
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}

impl crate::GetHeight for TextCursor {
  fn get_height(&self) -> u16 {
    self.matrix.get_height()
  }
}

impl crate::GetHeight for Frame {
  fn get_height(&self) -> u16 {
    self.border_rect.height()
  }
}

pub const TAB: u8 = 0;
pub const DLG: u8 = 1;
pub const MSG: u8 = 2;

pub struct TextBox {
  pub max:    Option<u16>,
  pub frame:  Option<Frame>,
  pub rect:   Rect,
  pub screen: ScreenCursor,
  pub text:   Rc<TextCursor>,
}

pub enum View {
  Layout(Rc<Layout>),
  TextBox(TextBox),
}

impl View {
  pub fn layout() {
  }

  pub fn framed_layout() {
  }

  pub fn text_cursor() {
  }

  pub fn framed_text_cursor() {
  }
}

pub struct Layout {
  pub rect:  Rect,
  pub max:   Option<u16>,
  pub frame: Option<Frame>,
  pub views: HashMap<u8, View>,
}

impl<T: GetRect> From<&T> for Layout {
  fn from(t: &T) -> Self {
    Self {
      rect: t.get_rect(),
      views: HashMap::default(),
    }
  }
}

impl Layout {
  // add
  pub fn add(&mut self, handle: u8, view: View) {
  }
  // remove
  pub fn remove(&mut self, handle: u8) {
  }
}
