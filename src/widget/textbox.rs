// src/widget/textbox.rs

use crate::{
  widget::{write_reset, ChangeType},
  screen::{Rect, PlaneView},
  text::{StyledText, StyledTextPlane, Style, Planar},
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo},
  style::{Print},
};
use std::io::{self, Write};

// coordinate Page and PlaneView
#[derive(Default)]
pub struct TextBox {
  pub style:   Style,
  pub rect:    Rect,
  pub content: StyledTextPlane,
  pub pos:     PlaneView,
  pub change:  Option<ChangeType>,
}
impl TextBox {
  pub fn new(text: Vec<StyledText>, rect: &Rect) -> Self {
    let content = StyledTextPlane::new(text, rect.w);
    let pos     = PlaneView::new(&rect);
    Self {
      rect:   rect.clone(),
      style:  Style::default(),
      change: Some(ChangeType::Scroll),
      pos, 
      content, 
    }
  }
  pub fn with_style(mut self, style: &Style) -> Self {
    self.style = style.clone();
    self
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.content.resize(self.rect.w);
    self.pos.resize(&self.content, &self.rect);
    self.reset_state();
  }
  pub fn y_len(&self) -> usize {
    self.content.y_len()
  }
  pub fn reset_state(&mut self) {
    self.change = Some(ChangeType::Scroll);
  }
  pub fn left(&mut self, step: usize) -> bool {
    if self.content.left(step) == 0 {
      self.change = self.pos.update(&self.content)
        .then_some(ChangeType::Scroll)
        .or(Some(ChangeType::Cursor));
      true
    } else {false}
  }
  pub fn right(&mut self, step: usize) -> bool {
    if self.content.right(step) == 0 {
      self.change = self.pos.update(&self.content)
        .then_some(ChangeType::Scroll)
        .or(Some(ChangeType::Cursor));
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    if self.content.down(step) {
      self.change = 
        self.pos.update(&self.content)
          .then_some(ChangeType::Scroll)
          .or(Some(ChangeType::Cursor));
      true
    } else {false}
  }
  pub fn up(&mut self, step: usize) -> bool {
    if self.content.up(step) {
      self.change = self.pos.update(&self.content)
        .then_some(ChangeType::Scroll)
        .or(Some(ChangeType::Cursor));
      true
    } else {false}
  }
  pub fn get_source_idx(&self) -> usize {
    self.content.get_source_idx()
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
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
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
        for x in self.rect.cut_x_range(len) {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    // render page space
    write_reset(writer)?;
    self.style.write(writer)?;
    if let Ok(len) = u16::try_from(lines.len()) {
      for y in self.rect.cut_y_range(len) {
        for x in self.rect.x_range() {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
  pub fn write_style<W>(&self, style: &Style, writer: &mut W) -> io::Result<()>
  where W: Write
  {
    write_reset(writer)?;
    style.write(writer)?;
    // render lines
    let lines = &self.content.text[self.pos.y_scroll()..];
    for (y, (idx, line)) in self.rect.y_range().zip(lines.iter()) {
      // render chars
      let x_scroll = 
        line.text.len().saturating_sub(1).min(self.pos.x_scroll());
      let chars = &line.text[x_scroll..];
      for (x, c) in self.rect.x_range().zip(chars.iter()) {
        writer.queue(MoveTo(x, y))?.queue(Print(c))?;
      }
      // render empty chars
      if let Ok(len) = u16::try_from(chars.len()) {
        for x in self.rect.cut_x_range(len) {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    // render empty lines
    if let Ok(len) = u16::try_from(lines.len()) {
      for y in self.rect.cut_y_range(len) {
        for x in self.rect.x_range() {
          writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
}
