// src/layout.rs

use crate::{
  Rect, GetRect,
  ScreenCursor,
};
use std::collections::HashMap;


pub const TAB:      u8 = 0;
pub const DLG_HEAD: u8 = 1;
pub const DLG_BODY: u8 = 2;
pub const MSG:      u8 = 3;

pub struct Window {
  cursor: ScreenCursor,
}

pub struct Layout {
  pub rect:     Rect,
  pub overlays: HashMap<u8, Window>,
}

impl<T: GetRect> From<&T> for Layout {
  fn from(t: &T) -> Self {
    Self {
      rect:     t.get_rect(),
      overlays: HashMap::default(),
    }
  }
}

impl Layout {
  // add
  pub fn add(&mut self, handle: &str, show: bool) {
  }
  // remove
  pub fn remove(&mut self, handle: &str) {
  }
}
