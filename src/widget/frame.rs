// src/widget/frame.rs

use crate::{
  common as c,
  widget::{write_reset, MarginSpec, BorderSpec},
  text::{Style},
  screen::{Rect},
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo},
  style::{Print},
};
use std::{
  io::{self, Write}
};

#[derive(Default)]
pub struct Frame {
  pub text_margin_spec:   MarginSpec,
  pub screen_margin_spec: MarginSpec,
  pub border_spec:        BorderSpec,
  pub margin_style:       Style,
  pub banner_style:       Style,
  pub border_rect:        Rect,
  pub outer_rect:         Rect,
  pub inner_rect:         Rect,
}
impl Frame {
  pub fn new(   screen:             &Rect, 
                border_spec:        BorderSpec, 
                screen_margin_spec: MarginSpec,
                text_margin_spec:   MarginSpec
                ) -> Self 
  {
    let border_rect = screen_margin_spec.get_rect(screen);
    let outer_rect  = screen_margin_spec.get_rect(screen).crop_x(1).crop_y(1);
    let inner_rect  = text_margin_spec.get_rect(&outer_rect);
    Self {
      margin_style: Style::default(),
      banner_style: Style::default(),
      border_rect,
      outer_rect,
      inner_rect,
      screen_margin_spec,
      text_margin_spec,
      border_spec,
    }
  }
  pub fn with_banner_style(mut self, style: &Style) -> Self {
    self.banner_style = style.clone();
    self
  }
  pub fn with_margin_style(mut self, style: &Style) -> Self {
    self.margin_style = style.clone();
    self
  }
  pub fn resize(&mut self, screen: &Rect) {
    self.border_rect = 
      self.screen_margin_spec.get_rect(screen);
    self.outer_rect = 
      self.screen_margin_spec.get_rect(screen).crop_x(1).crop_y(1);
    self.inner_rect = 
      self.text_margin_spec.get_rect(&self.outer_rect);
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    self.write_frame(writer)?;
    Ok(())
  }
  pub fn write_frame<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    // border
    write_reset(writer)?;
    self.border_spec.style.write(writer)?;
    let (ax, ay) = self.border_rect.a();
    let (bx, by) = self.border_rect.b();
    let (cx, cy) = self.border_rect.c();
    let (dx, dy) = self.border_rect.d();
    writer
      .queue(MoveTo(ax, ay))?.queue(Print(self.border_spec.a))?
      .queue(MoveTo(bx, by))?.queue(Print(self.border_spec.b))?
      .queue(MoveTo(cx, cy))?.queue(Print(self.border_spec.c))?
      .queue(MoveTo(dx, dy))?.queue(Print(self.border_spec.d))?;
    for x in self.border_rect.cropped_x_range(1) {
      writer
        .queue(MoveTo(x, ay))?.queue(Print(c::X_LINE))?
        .queue(MoveTo(x, cy))?.queue(Print(c::X_LINE))?;
    }
    for y in self.border_rect.cropped_y_range(1) {
      writer
        .queue(MoveTo(ax, y))?.queue(Print(c::Y_LINE))?
        .queue(MoveTo(bx, y))?.queue(Print(c::Y_LINE))?;
    }
    // margin
    write_reset(writer)?;
    self.margin_style.write(writer)?;
    for x in self.outer_rect.x_range() {
      for y in self.outer_rect.north_range(&self.inner_rect) {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for y in self.outer_rect.south_range(&self.inner_rect) {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    for y in self.inner_rect.y_range() {
      for x in self.outer_rect.east_range(&self.inner_rect) {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for x in self.outer_rect.west_range(&self.inner_rect) {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    write_reset(writer)?;
    Ok(())
  }
  pub fn write_banner<W>(&self, text: &str, writer: &mut W) -> io::Result<()>
  where W: Write
  {
    self.border_spec.style.write(writer)?;
    let y = self.border_rect.y;
    let mut x_range = self.inner_rect.x_range();
    writer
      .queue(MoveTo(x_range.start, y))?
      .queue(Print(self.border_spec.open))?;
    x_range.start += 1;
    self.banner_style.write(writer)?;
    let chars: Vec<char> = text.chars().collect();
    let range_len = x_range.len();
    let chars_len = chars.len();
    for (x, c) in x_range.clone().zip(chars.iter()) {
      writer.queue(MoveTo(x, y))?.queue(Print(c))?;
    }
    // close bracket before limit
    if chars_len < range_len {
      x_range.start += u16::try_from(chars_len)
          .expect("We do not have Allah's permission");
      write_reset(writer)?;
      self.border_spec.style.write(writer)?;
      writer
        .queue(MoveTo(x_range.start, y))?
        .queue(Print(self.border_spec.close))?;
      x_range.start += 1;
      for x in x_range {
        writer.queue(MoveTo(x, y))?.queue(Print(c::X_LINE))?;
      }
    // close bracket at limit
    } else {
      write_reset(writer)?;
      self.border_spec.style.write(writer)?;
      writer.queue(MoveTo(x_range.end - 1, y))?
        .queue(Print(self.border_spec.close))?;
    }
    write_reset(writer)?;
    Ok(())
  }
}
