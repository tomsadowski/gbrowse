// src/widget.rs

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
pub trait Linear<T> {
  fn items(&self) -> &Vec<T>;
  fn head(&self) -> usize;
  fn head_mut(&mut self) -> &mut usize;
  fn max_head(&self) -> usize;
  
  fn current(&self) -> &T {
    &self.items()[self.head()]
  }
  fn window(&self, shift: usize, length: u16) -> std::iter::Take<std::slice::Iter<'_, T>> {
    let shift = std::cmp::min(shift, self.items().len().saturating_sub(1));
    self.items()[shift..].iter().take(length.into()) 
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
pub trait LinearMut<T>: Linear<T> {
  fn items_mut(&mut self) -> &mut Vec<T>;

  fn delete(&mut self) -> bool {
    let head = self.head();
    if head < self.items().len() {
      self.items_mut().remove(head);
      true
    } else {false}
  }
  fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.backward(1);
      let head = self.head();
      self.items_mut().remove(head);
      true
    } else {false}
  }
  fn insert(&mut self, c: T) -> bool {
    let head = self.head();
    if head + 1 == self.items().len() || self.items().len() == 0 {
      self.items_mut().push(c);
      self.forward(1);
      true
    } else {
      self.items_mut().insert(head, c);
      self.forward(1);
      true
    }
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
