// src/layout.rs

use crate::{
  Rect,
  ScreenCursor,
};
use std::collections::HashMap;


pub struct Layout {
  pub rect:     Rect,
  pub base:     ScreenCursor,
  pub overlays: HashMap<String, ScreenCursor>,
}

impl Layout {
  // add
  // remove
}
