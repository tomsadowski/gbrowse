// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  Cursor, 
  TextBox,
  Draw,
};


pub enum Direction {
  NorthSouth,
  SouthNorth,
  WestEast, 
  EastWest,
}

pub struct ViewStack<'a, T> {
  pub view:        Rect,
  pub windows:     Cursor<&'a T>,
  pub orientation: Direction,
}

impl<'a, T, V> From<V> for ViewStack<'a, T> 
where &'a T: Default,
      V:     ViewPort,
{
  fn from(view: V) -> Self {
    Self {
      view:        view.get_view_port(),
      windows:     Cursor::default(),
      orientation: Direction::NorthSouth,
    }
  }
}

impl<'a, T> ViewStack<'a, T> {
  fn get_weight(&self, window: &TextBox) -> usize {
    match self.orientation {
      Direction::WestEast | Direction::EastWest => 
        window.get_view_port().w.into(),
      Direction::NorthSouth | Direction::SouthNorth => 
        window.get_view_port().h.into(),
    }
  }
}

impl<'a, T: Draw> Draw for ViewStack<'a, T> {
  fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    for textbox in self.windows.iter() {
      textbox.draw(writer)?;
    }
    Ok(())
  }
}
