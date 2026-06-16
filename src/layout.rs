// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  IndexedCursor, 
  TextBox,
};


pub enum Direction {
  NorthSouth,
  SouthNorth,
  WestEast, 
  EastWest,
}

pub struct ViewStack {
  pub view:        Rect,
  pub windows:     IndexedCursor<Rect>,
  pub orientation: Direction,
}

impl<V: ViewPort> From<V> for ViewStack {
  fn from(view: V) -> Self {
    Self {
      windows: {
        let mut c = IndexedCursor::default();
        c.insert(view.get_view_port());
        c
      },
      view:        view.get_view_port(),
      orientation: Direction::NorthSouth,
    }
  }
}

impl ViewStack {
  fn get_weight(&self, window: &TextBox) -> usize {
    match self.orientation {
      Direction::WestEast | Direction::EastWest => 
        window.get_view_port().w.into(),
      Direction::NorthSouth | Direction::SouthNorth => 
        window.get_view_port().h.into(),
    }
  }
}
