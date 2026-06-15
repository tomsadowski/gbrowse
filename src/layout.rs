// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  Cursor, 
  TextBox,
};


pub enum Orientation {
  Horizontal, Vertical,
}

pub struct Layout {
  pub view:        Rect,
  pub head:        usize,
  pub windows:     Cursor<TextBox>,
  pub orientation: Orientation,
}

impl<V: ViewPort> From<V> for Layout {
  fn from(view: V) -> Self {
    Self {
      view:        view.get_view_port(),
      head:        0,
      windows:     Cursor::default(),
      orientation: Orientation::Horizontal,
    }
  }
}

impl Layout {
  fn get_weight(&self, window: &TextBox) -> usize {
    match self.orientation {
      Orientation::Horizontal => 
        window.get_view_port().w.into(),
      Orientation::Vertical => 
        window.get_view_port().h.into(),
    }
  }
}
