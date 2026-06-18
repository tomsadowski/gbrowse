// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  IndexedCursor, 
  TextBox,
};


pub struct Layout {
  pub view: Rect,
  pub base: Rect,
  pub temp: IndexedCursor<Rect>,
}

impl<V: ViewPort> From<V> for Layout {
  fn from(view: V) -> Self {
    Self {
      temp: {
        let mut c = IndexedCursor::default();
        c.insert(view.get_view_port());
        c
      },
      base: view.get_view_port(),
      view: view.get_view_port(),
    }
  }
}

impl Layout {
  pub fn update_base(&mut self) {

  }
  pub fn size_base(&self, textbox: &mut TextBox) {
    textbox.resize(self.base)
  }
}
