// src/widget/frame.rs

use crate::{
  common as c,
  user::{MarginSpec, BorderSpec},
  widget::{Rect, Style},
};
use crossterm::{
  QueueableCommand, 
  cursor::{position, MoveTo, MoveLeft, MoveUp, MoveDown, MoveRight},
  style::{Print, SetAttribute, Attribute},
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
    self.border_rect = self.screen_margin_spec.get_rect(screen);
    self.outer_rect  = self.screen_margin_spec.get_rect(screen).crop_x(1).crop_y(1);
    self.inner_rect  = self.text_margin_spec.get_rect(&self.outer_rect);
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    self.write_frame(writer)?;
    Ok(())
  }
  pub fn write_frame<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    // border
    let (ax, ay) = self.border_rect.a();
    let (bx, by) = self.border_rect.b();
    let (cx, cy) = self.border_rect.c();
    let (dx, dy) = self.border_rect.d();
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.border_spec.style)?
      .queue(MoveTo(ax, ay))?.queue(Print(self.border_spec.a))?
      .queue(MoveTo(bx, by))?.queue(Print(self.border_spec.b))?
      .queue(MoveTo(cx, cy))?.queue(Print(self.border_spec.c))?
      .queue(MoveTo(dx, dy))?.queue(Print(self.border_spec.d))?;
    for x in self.border_rect.cropped_x(1).x_range() {
      writer
        .queue(MoveTo(x, ay))?.queue(Print(c::X_LINE))?
        .queue(MoveTo(x, cy))?.queue(Print(c::X_LINE))?;
    }
    for y in self.border_rect.cropped_y(1).y_range() {
      writer
        .queue(MoveTo(ax, y))?.queue(Print(c::Y_LINE))?
        .queue(MoveTo(bx, y))?.queue(Print(c::Y_LINE))?;
    }
    // margin
    writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.margin_style)?;
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
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
  pub fn write_footer<W: Write>(&self, text: &str, style: &Style, writer: &mut W) 
    -> io::Result<()> 
  {
    let mut x = self.inner_rect.x_end().saturating_sub(1);
    let     y = self.border_rect.y_end().saturating_sub(1);
    writer
      .queue(MoveTo(x, y))?
      .queue(&self.border_spec.style)?
      .queue(Print(self.border_spec.close))?
      .queue(MoveLeft(2))?
      .queue(Print(' '))?
      .queue(&self.banner_style)?;
    x -= 2;
    for c in text.chars().rev().take(self.inner_rect.cropped_x(2).w.into()) {
      writer.queue(MoveLeft(2))?.queue(Print(c))?;
      x -= 1;
    }
    writer
      .queue(MoveLeft(2))?
      .queue(Print(' '))?
      .queue(&self.border_spec.style)?
      .queue(MoveLeft(2))?
      .queue(Print(self.border_spec.open))?;
    x -= 2;
    for _ in self.inner_rect.x..x {
      writer.queue(MoveLeft(2))?.queue(Print(c::X_LINE))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
  pub fn write_banner<W: Write>(&self, text: &str, writer: &mut W) -> io::Result<()> {
    let mut x = self.inner_rect.x;
    let     y = self.border_rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(&self.border_spec.style)?
      .queue(Print(self.border_spec.open))?
      .queue(Print(' '))?
      .queue(&self.banner_style)?;
    x += 2;
    for c in text.chars().take(self.inner_rect.cropped_x(2).w.into()) {
      writer.queue(Print(c))?;
      x += 1;
    }
    writer
      .queue(&self.border_spec.style)?
      .queue(Print(' '))?
      .queue(Print(self.border_spec.close))?;
    x += 2;
    for _ in x..self.inner_rect.x_end() {
      writer.queue(Print(c::X_LINE))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
