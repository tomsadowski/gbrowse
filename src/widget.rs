// src/widget/mod.rs

mod frame;
mod textbox;
mod editbox;
mod rect;
mod planeview;

pub use self::frame::Frame;
pub use self::textbox::TextBox;
pub use self::editbox::EditBox;
pub use self::rect::Rect;
pub use self::planeview::{LineView, PlaneView};

use crate::{
  common as c,
  text::{Style}, 
};
use crossterm::{
  QueueableCommand, 
  style::{SetAttribute, Attribute},
  cursor::{self, MoveTo},
};
use std::{
  io::{self, Write}
};


pub fn write_reset<W: Write>(writer: &mut W) -> io::Result<()> {
  writer.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}
pub fn cursor_hide<W: Write>(writer: &mut W) -> io::Result<()> {
  writer.queue(cursor::Hide)?;
  Ok(())
}
pub trait PlaneWidget {
  fn pos(&self) -> (u16, u16);
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()>;

  fn write_cursor<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let (x, y) = self.pos();
    writer.queue(MoveTo(x, y))?.queue(cursor::Show)?;
    Ok(())
  }
}
impl PlaneWidget for TextBox {
  fn pos(&self) -> (u16, u16) {
    (self.pos.x_cursor(), self.pos.y_cursor())
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
}
impl PlaneWidget for EditBox {
  fn pos(&self) -> (u16, u16) {
    (self.pos.cursor(), self.rect.y)
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MarginSpec {
  pub north: u16,
  pub south: u16,
  pub east:  u16,
  pub west:  u16,
}
impl Default for MarginSpec {
  fn default() -> Self {
    Self {north: 0, south: 0, east: 0, west: 0}
  }
}
impl MarginSpec {
  pub fn get_rect(&self, screen: &Rect) -> Rect {
    screen.clone()
      .crop_north(self.north).crop_south(self.south)
      .crop_east(self.east).crop_west(self.west)
  }
}
#[derive(Debug, Clone)]
pub struct BorderSpec {
  pub style: Style,
  pub a:     char,
  pub b:     char,
  pub c:     char,
  pub d:     char,
  pub open:  char,
  pub close: char,
}
impl Default for BorderSpec {
  fn default() -> Self {
    Self {
      style: Style::default(),
      a:     c::A_SQR,
      b:     c::B_SQR,
      c:     c::C_SQR,
      d:     c::D_SQR,
      open:  c::OPEN_SQR,
      close: c::CLOSE_SQR,
    }
  }
}
