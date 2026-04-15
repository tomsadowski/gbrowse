// src/widget/dynamo.rs

use crate::{
  widget::{write_reset, ChangeType},
  screen::{Rect, PlaneView},
  text::{Style, StyledText, StyledTextPlane, Planar},
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
pub struct Dynamo {
  pub style:   Style,
  pub change:  Option<ChangeType>,
  pub limit:   Rect,
  pub used:    Rect,
  pub content: StyledTextPlane,
  pub pos:     PlaneView,
  pub write_unused_x: bool,
}
impl Dynamo {
  pub fn new(text: Vec<StyledText>, limit: &Rect) -> Self {
    let content = StyledTextPlane::new(text, limit.w);
    let used = 
      if let Ok(h) = u16::try_from(content.y_len()) {
        limit.limit_h(h)
      } else {
        limit.clone()
      };
    let pos = PlaneView::new(&used);
    Self {
      style: Style::default(),
      limit: limit.clone(),
      change: Some(ChangeType::Scroll),
      write_unused_x: false,
      used,
      pos, 
      content, 
    }
  }
  pub fn resize(&mut self, limit: &Rect) {
    self.limit = limit.clone();
    self.content.resize(self.limit.w);
    let used = 
      if let Ok(h) = u16::try_from(self.content.y_len()) {
        self.limit.limit_h(h)
      } else {
        self.limit.clone()
      };
    self.pos.resize(&self.content, &self.used);
    self.reset_state();
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
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    write_reset(writer)?;
    self.style.write(writer)?;
    // render lines
    let y_scroll = self.pos.y_scroll();
    let lines    = &self.content.text[y_scroll..];
    for (y, (idx, line)) in self.used.y_range().zip(lines.iter()) {
      // render chars
      self.content.source[*idx].style.write(writer)?;
      let x_scroll = line.text.len().saturating_sub(1).min(self.pos.x_scroll());
      let chars    = &line.text[x_scroll..];
      for (x, c) in self.used.x_range().zip(chars.iter()) {
        writer.queue(MoveTo(x, y))?.queue(Print(c))?;
      }
      // render line space
      if self.write_unused_x {
        write_reset(writer)?;
        self.style.write(writer)?;
        if let Ok(len) = u16::try_from(chars.len()) {
          for x in self.used.cut_x_range(len) {
            writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
          }
        }
      }
    }
    write_reset(writer)?;
    Ok(())
  }
}
