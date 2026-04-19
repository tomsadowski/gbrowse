// src/editbox.rs

use crate::{
  widget::{write_reset},
  screen::{Rect, LineView},
  text::{Style, EditLine, Linear},
};
use crossterm::{
  QueueableCommand,
  style::Print,
  cursor::{self, MoveTo},
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
  pub pos:            LineView,
  pub write_unused_x: bool,
}
impl EditBox {
  pub fn new(rect: &Rect) -> Self {
    let content = EditLine::from("");
    let pos     = LineView::new(rect.x, rect.w);
    Self {
      rect:             rect.clone(),
      style:            Style::default(),
      write_unused_x:   false,
      write:            true,
      pos, 
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
    self.pos.resize(self.content.head, self.rect.x, self.rect.w);
    self.reset_state();
  }
  pub fn reset_state(&mut self) {
    self.write = true;
  }
  pub fn left(&mut self, step: usize) -> bool {
    if self.content.backward(step) == 0 {
      self.write = self.pos.update(self.content.head);
      true
    } else {false}
  }
  pub fn right(&mut self, step: usize) -> bool {
    if self.content.forward(step) == 0 {
      self.write = self.pos.update(self.content.head);
      true
    } else {false}
  }
  pub fn delete(&mut self) -> bool {
    if self.content.delete() {
      self.write_unused_x = true;
      self.pos.update(self.content.head);
      self.write = true;
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.content.backspace() {
      self.write_unused_x = true;
      self.pos.update(self.content.head);
      self.write = true;
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.content.insert(c) {
      self.pos.update(self.content.head);
      self.write = true;
      true
    } else {false}
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    write_reset(writer)?;
    self.style.write(writer)?;
    // render chars
    let scroll = self.pos.scroll();
    let text   = &self.content.text[scroll..];
    for (x, c) in self.rect.x_range().zip(text.chars()) {
      writer.queue(MoveTo(x, self.rect.y))?.queue(Print(c))?;
    }
    // render page space
    if self.write_unused_x {
      write_reset(writer)?;
      self.style.write(writer)?;
      if let Ok(len) = u16::try_from(text.len()) {
        for x in self.rect.cropped_west_range(len) {
          writer.queue(MoveTo(x, self.rect.y))?.queue(Print(' '))?;
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
}
