// src/widget/textbox.rs

use crate::{
  widget::{Rect, PlaneView, write_reset, StyledText, StyledTextPlane, Style, Planar},
};
use crossterm::{
  QueueableCommand, 
  cursor::{position, MoveTo, MoveDown, MoveRight, MoveLeft, MoveUp, MoveToColumn},
  style::{Print},
};
use std::{
  io::{self, Write},
};

#[derive(Default)]
pub struct TextBox {
  pub rect:           Rect,
  pub style:          Style,
  pub content:        StyledTextPlane,
  pub pos:            PlaneView,
  pub write:          bool,
  pub write_unused_x: bool,
  pub write_unused_y: bool,
}
impl TextBox {
  pub fn new(text: Vec<StyledText>, rect: &Rect) -> Self {
    let content = StyledTextPlane::new(text, rect.w);
    let pos     = PlaneView::new(&rect);
    Self {
      write_unused_x: true,
      write_unused_y: true,
      style: Style::default(),
      write: true,
      rect:   rect.clone(),
      pos, content,
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
  pub fn y_len(&self) -> usize {
    self.content.y_len()
  }
  pub fn get_source_idx(&self) -> usize {
    self.content.get_source_idx()
  }
  pub fn get_source(&self) -> String {
    self.content.get_source()
  }
  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.content.y_len()) {
      self.rect.limit_h(h)
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
    self.pos.resize(&self.content, &rect);
    self.reset_state();
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.content.resize(rect.w);
    self.pos.resize(&self.content, &rect);
    self.reset_state();
  }
  pub fn left(&mut self, step: usize) -> bool {
    if self.content.left(step) == 0 {
      self.write = self.pos.update(&self.content);
      true
    } else {false}
  }
  pub fn right(&mut self, step: usize) -> bool {
    if self.content.right(step) == 0 {
      self.write = self.pos.update(&self.content);
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    if self.content.down(step) {
      self.write = self.pos.update(&self.content);
      true
    } else {false}
  }
  pub fn up(&mut self, step: usize) -> bool {
    if self.content.up(step) {
      self.write = self.pos.update(&self.content);
      true
    } else {false}
  }
  pub fn clear<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    write_reset(writer)?;
    self.style.write(writer)?;
    for y in self.rect.y_range() {
      for x in self.rect.x_range() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    write_reset(writer)?;
    Ok(())
  }
  pub fn write_full<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    write_reset(writer)?;
    self.style.write(writer)?;
    // render lines
    let lines = &self.content.text[self.pos.y_scroll()..];
    for (y, (idx, line)) in self.rect.y_range().zip(lines.iter()) {
      // render chars
      self.content.source[*idx].style.write(writer)?;
      let x_scroll = 
        line.text.len().saturating_sub(1).min(self.pos.x_scroll());
      let chars = &line.text[x_scroll..];
      for (x, c) in self.rect.x_range().zip(chars.iter()) {
        writer.queue(MoveTo(x, y))?.queue(Print(c))?;
      }
      // render line space
      write_reset(writer)?;
      self.style.write(writer)?;
      if let Ok(len) = u16::try_from(chars.len()) {
        for x in self.rect.cropped_west_range(len) {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    // render page space
    write_reset(writer)?;
    self.style.write(writer)?;
    if let Ok(len) = u16::try_from(lines.len()) {
      for y in self.rect.cropped_north_range(len) {
        for x in self.rect.x_range() {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
  pub fn write_style<W: Write>(&self, style: &Style, writer: &mut W) -> io::Result<()> {
    write_reset(writer)?;
    style.write(writer)?;
    // render lines
    let lines = &self.content.text[self.pos.y_scroll()..];
    for (y, (_, line)) in self.rect.y_range().zip(lines.iter()) {
      // render chars
      let x_scroll = self.pos.x_scroll().min(line.text.len().saturating_sub(1));
      let chars = &line.text[x_scroll..];
      for (x, c) in self.rect.x_range().zip(chars.iter()) {
        writer.queue(MoveTo(x, y))?.queue(Print(c))?;
      }
      if self.write_unused_x {
        // render empty chars
        if let Ok(len) = u16::try_from(chars.len()) {
          for x in self.rect.cropped_west_range(len) {
            writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
          }
        }
      }
    }
    // render empty lines
    if self.write_unused_y {
      if let Ok(len) = u16::try_from(lines.len()) {
        for y in self.rect.cropped_north_range(len) {
          for x in self.rect.x_range() {
            writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
          }
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let mut y = self.rect.y;
    writer.queue(MoveTo(x, y))?;
    write_reset(writer)?;
    self.style.write(writer)?;
    let y_scroll = self.pos.y_scroll();
    // render lines
    for (_, (idx, line)) in self.rect.y_range().zip(self.content.text[y_scroll..].iter()) 
    {
      // render chars
      self.content.source[*idx].style.write(writer)?;
      let x_scroll = self.pos.x_scroll().min(line.text.len().saturating_sub(1));
      for (_, c) in self.rect.x_range().zip(line.text[x_scroll..].iter()) {
        writer.queue(Print(c))?;
        x += 1;
      }
      // render line space
      if self.write_unused_x {
        write_reset(writer)?;
        self.style.write(writer)?;
        for _ in x..self.rect.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.rect.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    // render empty lines
    if self.write_unused_y {
      write_reset(writer)?;
      self.style.write(writer)?;
      for _ in y..self.rect.y_end() {
        for _ in self.rect.x_range() {
          writer.queue(Print(' '))?;
        }
        x = self.rect.x;
        y += 1;
        writer.queue(MoveTo(x, y))?;
      }
    }
    write_reset(writer)?;
    Ok(())
  }
}
