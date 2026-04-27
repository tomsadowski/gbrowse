// src/widget/mod.rs

mod frame;
mod text;
mod textbox;
mod editbox;
mod rect;
mod planeview;

pub use self::frame::Frame;
pub use self::textbox::TextBox;
pub use self::editbox::EditBox;
pub use self::rect::Rect;
pub use self::planeview::{LineView, PlaneView};
pub use self::text::{TextLine, EditLine, Style, StyledText, StyledTextPlane};

use crate::{
  common as c,
};
use crossterm::{
  QueueableCommand, 
  style::{SetAttribute, Attribute},
  cursor::{self, MoveTo},
};
use std::{
  io::{self, Write},
};

pub fn reset() -> SetAttribute {
  SetAttribute(Attribute::Reset)
}
pub fn cursor_hide<W: Write>(writer: &mut W) -> io::Result<()> {
  writer.queue(cursor::Hide)?;
  Ok(())
}
pub trait LinearList<T> {
  fn get_items(&self) -> &Vec<T>;

  fn current(&self, scroll: usize, screen: u16) -> std::iter::Take<std::slice::Iter<'_, T>> {
    let scroll = scroll.min(self.get_items().len().saturating_sub(1));
    self.get_items()[scroll..].iter().take(screen.into()) 
  }
}
pub trait Linear {
  fn len(&self) -> usize;
  fn head(&self) -> usize;
  fn head_mut(&mut self) -> &mut usize;


  fn max_head(&self) -> usize {
    self.len().saturating_sub(1)
  }
  fn fit(&mut self, new_cursor: usize) {
    *self.head_mut() = self.max_head().min(new_cursor);
  }
  fn start(&mut self) {
    *self.head_mut() = 0;
  }
  fn end(&mut self) {
    *self.head_mut() = self.max_head();
  }
  fn peek_backward(&self, step: usize) -> usize {
    if step > self.head() {
      step - self.head()
    } else {0}
  }
  fn peek_forward(&self, step: usize) -> usize {
    let max_head = self.max_head();
    if self.head() + step > max_head {
      self.head() + step - max_head
    } else {0}
  }
  fn backward(&mut self, mut step: usize) -> usize {
    if step > self.head() {
      step -= self.head();
      *self.head_mut() = 0;
      step
    } else {
      *self.head_mut() -= step;
      0
    }
  }
  fn forward(&mut self, mut step: usize) -> usize {
    if self.head() + step > self.max_head() {
      step = self.head() + step - self.max_head();
      *self.head_mut() = self.max_head();
      step
    } else {
      *self.head_mut() += step;
      0
    }
  }
  fn wrapping_backward(&mut self, step: usize) {
    if step > self.head() {
      self.end();
    } else {
      *self.head_mut() -= step;
    }
  }
  fn wrapping_forward(&mut self, step: usize) {
    if self.head() + step > self.max_head() {
      self.start();
    } else {
      *self.head_mut() += step;
    }
  }
}
pub trait Planar {
  fn x_len(&self) -> usize;
  fn x_head(&self) -> usize;
  fn y_len(&self) -> usize;
  fn y_head(&self) -> usize;
  fn y_head_mut(&mut self) -> &mut usize;
}
impl<P: Planar> Linear for P {
  fn len(&self) -> usize {
    self.y_len()
  }
  fn head(&self) -> usize {
    self.y_head()
  }
  fn head_mut(&mut self) -> &mut usize {
    self.y_head_mut()
  }
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
