// src/editbox.rs

use crate::{
  widget::{Rect, ScreenCursor, Style, EditLine, UnitCursor, UnitCursorMut},
};
use crossterm::{
  QueueableCommand,
  style::{Print, SetAttribute, Attribute},
  cursor::MoveTo,
};
use unicode_width::UnicodeWidthChar;
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
  pub cursor:         ScreenCursor,
  pub write_unused_x: bool,
}
impl EditBox {
  pub fn new(rect: &Rect) -> Self {
    let content = EditLine::from("");
    let rect    = rect.top_row();
    let pos     = ScreenCursor::new(&rect);
    Self {
      rect:           rect.clone(),
      style:          Style::default(),
      write_unused_x: false,
      write:          true,
      cursor: pos, 
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
    self.rect = rect.top_row();
    self.cursor.x.resize(&self.content, self.rect.x, self.rect.w);
    self.reset_state();
  }
  pub fn reset_state(&mut self) {
    self.write = true;
  }
  pub fn left(&mut self, delta: usize) -> bool {
    if self.content.backward(delta) == 0 {
      self.write = self.cursor.x.update(&self.content);
      true
    } else {false}
  }
  pub fn right(&mut self, delta: usize) -> bool {
    if self.content.forward(delta) == 0 {
      self.write = self.cursor.x.update(&self.content);
      true
    } else {false}
  }
  pub fn delete(&mut self) -> bool {
    if self.content.delete() {
      self.write_unused_x = true;
      self.cursor.x.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.content.backspace() {
      self.write_unused_x = true;
      self.cursor.x.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.content.insert(c) {
      self.cursor.x.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
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
      .iter_from(self.cursor.x_scroll())
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
