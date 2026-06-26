// src/layout.rs

use crate::{
  Rect, 
  GetRect,
  Frame,
  ScreenCursor,
  TextBox,
};
use std::{
  collections::HashMap,
  rc::Rc,
};


pub trait GetHeight { fn get_height(&self) -> u16; }

impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 { 
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}

pub const TAB: u8 = 0;
pub const DLG: u8 = 1;
pub const MSG: u8 = 2;

pub enum Max {
  Size(u16),
  Fill,
}

pub enum View {
  Layout         (Rc<Layout>),
  FramedLayout   (Rc<Layout>, Frame),
  TextBox        (ScreenCursor, Rc<TextBox>),
  FramedTextBox  (ScreenCursor, Rc<TextBox>, Frame),
}

pub struct ViewParams {
  max:  Max,
  view: View,
}

pub struct Layout {
  pub rect:  Rect,
  pub views: HashMap<u8, ViewParams>,
}

impl<T: GetRect> From<&T> for Layout {
  fn from(t: &T) -> Self {
    Self {
      rect:  t.get_rect(),
      views: HashMap::default(),
    }
  }
}

impl Layout {
  // add
  pub fn add(&mut self, handle: &str, view: View) {
  }
  // remove
  pub fn remove(&mut self, handle: &str) {
  }
}
