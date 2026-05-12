// src/widget/textbox.rs

use crate::{
  widget::{Rect, DataCursor, ScreenCursor, StyledText, StyledTextPlane, Style, PlaneWidget},
};
use crossterm::{
  QueueableCommand, 
  cursor::{position, MoveTo, MoveDown, MoveRight, MoveLeft, MoveUp, MoveToColumn},
  style::{Print, SetAttribute, Attribute},
};
use std::{
  io::{self, Write},
};

#[derive(Default)]
pub struct TextBox {
  pub rect:           Rect,
  pub style:          Style,
  pub content:        StyledTextPlane,
  pub cursor:         ScreenCursor,
  pub write:          bool,
  pub write_unused_x: bool,
  pub write_unused_y: bool,
}
impl PlaneWidget for TextBox {
  fn pos(&self) -> (u16, u16) {
    (self.cursor.x_cursor(), self.cursor.y_cursor())
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
}
impl TextBox {
  pub fn new(text: Vec<StyledText>, rect: &Rect) -> Self {
    let content = StyledTextPlane::new(text, rect.w);
    let pos     = ScreenCursor::new(&rect);
    Self {
      write_unused_x: true,
      write_unused_y: true,
      style:          Style::default(),
      write:          true,
      rect:           rect.clone(),
      cursor:         pos, 
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
  pub fn write_unused_y(mut self, write: bool) -> Self {
    self.write_unused_y = write;
    self
  }
  pub fn write_unused(mut self, write: bool) -> Self {
    self.write_unused_x = write;
    self.write_unused_y = write;
    self
  }
  pub fn get_source_idx(&self) -> usize {
    self.content.current().idx
  }
  pub fn get_source(&self) -> String {
    self.content.get_source()
  }
  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.content.data().len()) {
      self.rect.clone().cap_height(h)
    } else {
      self.rect.clone()
    }
  }
  pub fn reset_state(&mut self) {
    self.write = true;
  }
  pub fn restyle(&mut self, text: Vec<StyledText>, rect: &Rect) {
    self.rect = rect.clone();
    self.content.restyle(text, rect.w);
    self.cursor.resize(&self.content, &rect);
    self.reset_state();
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.content.resize(rect.w);
    self.cursor.resize(&self.content, &rect);
    self.reset_state();
  }
  pub fn left(&mut self, delta: usize) -> bool {
    if self.content.left(delta) == 0 {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn right(&mut self, delta: usize) -> bool {
    if self.content.right(delta) == 0 {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn down(&mut self, delta: usize) -> bool {
    if self.content.down(delta) {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn up(&mut self, delta: usize) -> bool {
    if self.content.up(delta) {
      self.write = self.cursor.update(&self.content);
      true
    } else {false}
  }
  pub fn clear<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    for y in self.rect.y_range() {
      for x in self.rect.x_range() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
  pub fn write_style<W: Write>(&self, style: &Style, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let mut y = self.rect.y;
    writer.queue(MoveTo(x, y))?.queue(SetAttribute(Attribute::Reset))?.queue(&style)?;
    for line in self.content.current_view(self.cursor.y_scroll()).take(self.rect.h.into()) {
      for c in line.current_view(self.cursor.x_scroll()).take(self.rect.w.into()) {
        writer.queue(Print(c))?;
        x += 1;
      }
      if self.write_unused_x {
        for _ in x..self.rect.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.rect.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let mut y = self.rect.y;
    writer.queue(MoveTo(x, y))?.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    for line in self.content.current_view(self.cursor.y_scroll()).take(self.rect.h.into()) {
      writer.queue(&self.content.source[line.idx].style)?;
      for c in line.current_view(self.cursor.x_scroll()).take(self.rect.w.into()) {
        writer.queue(Print(c))?;
        x += 1;
      }
      if self.write_unused_x {
        writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
        for _ in x..self.rect.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.rect.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    if self.write_unused_y {
      writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
      for _ in y..self.rect.y_end() {
        for _ in self.rect.x_range() {
          writer.queue(Print(' '))?;
        }
        x = self.rect.x;
        y += 1;
        writer.queue(MoveTo(x, y))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
