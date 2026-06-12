// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  UnitCursor, 
  UnitCursorMut, 
  TextBox,
  EditLine,
  TextLine,
};


pub enum Window {
  Edit(TextBox<EditLine>),
  Text(TextBox<TextLine>),
}

impl ViewPort for Window {
  fn get_view_port(&self) -> Rect {
    match self {
      Window::Edit(textbox) => textbox.view,
      Window::Text(textbox) => textbox.view,
    }
  }
}

pub enum Orientation {
  Horizontal, Vertical,
}

pub struct Layout {
  pub view:    Rect,
  pub head:    usize,
  pub windows: Vec<Window>,
  pub orientation: Orientation,
}

impl UnitCursor for Layout {
  type Unit = Window;
  fn get_units(&self) -> &Vec<Self::Unit> {
    &self.windows
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
    self.windows.len().saturating_sub(1)
  }
}

impl UnitCursorMut for Layout {
  fn units_mut(&mut self) -> &mut Vec<Window> {
    &mut self.windows
  }
}

impl<V: ViewPort> From<V> for Layout {
  fn from(view: V) -> Self {
    Self {
      view:        view.get_view_port(),
      head:        0,
      windows:     vec![],
      orientation: Orientation::Horizontal,
    }
  }
}

impl Layout {
  fn get_weight(&self) -> usize {
    match self.orientation {
      Orientation::Horizontal => 0,
      Orientation::Vertical => 0
    }
  }
}
