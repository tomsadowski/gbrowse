// src/layout.rs

use crate::{
  Rect, 
  GetRect,
  Frame,
  ScreenCursor,
};
use std::collections::HashMap;


pub const TAB: u8 = 0;
pub const DLG: u8 = 1;
pub const MSG: u8 = 2;

pub enum Max {
  Size(u16),
  Fill,
}

pub enum ViewType {
  Cursor(ScreenCursor),
  Screen(Rect),
}

pub struct View {
  frame:    Option<Frame>,
  max:      Max,
  viewtype: ViewType,
}

pub struct Layout {
  pub rect:  Rect,
  pub views: HashMap<u8, View>,
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
