// src/editbox.rs

use crate::{
  widget::{Rect, SizeCursorView, Style, EditLine, UnitCursor, UnitCursorMut, PlaneWidget},
};
use crossterm::{
  QueueableCommand,
  style::{Print, SetAttribute, Attribute},
  cursor::MoveTo,
};
use std::{
  io::{self, Write}
};

// coordinate Page and PlaneView
#[derive(Default)]
pub struct EditBox {
  pub style:          Style,
  pub write:          bool,
  pub rect:           Rect,
  pub content:        EditLine,
  pub cursor:         SizeCursorView,
  pub write_unused_x: bool,
}
impl PlaneWidget for EditBox {
  fn pos(&self) -> (u16, u16) {
    (self.cursor.cursor(), self.rect.y)
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
}
impl EditBox {
  pub fn new(rect: &Rect) -> Self {
    let content = EditLine::from("");
    let cursor  = SizeCursorView::new(rect.x, rect.w);
    Self {
      rect:           rect.clone(),
      style:          Style::default(),
      write_unused_x: false,
      write:          true,
      cursor, 
      content, 
    }
  }
  pub fn with_style(mut self, style: &Style) -> Self {
    self.style = style.clone();
    self
  }
  pub fn write_unused_x(mut self, write: bool) -> Self {
    self.write_unused_x = write;
    self
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.cursor.resize(&self.content, self.rect.x, self.rect.w);
    self.reset_state();
  }
  pub fn reset_state(&mut self) {
    self.write = true;
  }
  pub fn left(&mut self, delta: usize) -> bool {
    if self.content.backward(delta) == 0 {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn right(&mut self, delta: usize) -> bool {
    if self.content.forward(delta) == 0 {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn delete(&mut self) -> bool {
    if self.content.delete() {
      self.write_unused_x = true;
      self.cursor.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.content.backspace() {
      self.write_unused_x = true;
      self.cursor.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.content.insert(c) {
      self.cursor.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let     y = self.rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    // render chars
    for c in self.content
      .iter_from(self.cursor.scroll())
      .take(self.rect.w.into()) 
    {
      writer.queue(Print(c))?;
      x += 1;
    }
    writer.queue(MoveTo(x, y))?;
    // render page space
    if self.write_unused_x {
      for _ in x..self.rect.x_end() {
        writer.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
