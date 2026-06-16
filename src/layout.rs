// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  Cursor, 
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
  pub windows:     Cursor<TextBox>,
  pub orientation: Direction,
}

impl<V: ViewPort> From<V> for ViewStack {
  fn from(view: V) -> Self {
    Self {
      view:        view.get_view_port(),
      windows:     Cursor::default(),
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

  pub fn write<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    for textbox in self.windows.iter() {
      textbox.write(writer, 0)?;
    }
    Ok(())
  }
}
