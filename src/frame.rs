// src/frame.rs

use crate::{
  ViewPort, 
  Rect, 
  Style, 
  Margins, 
  BorderStyle,
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use std::io::Write;


#[derive(Copy, Default, Clone)]
pub struct Frame {
  pub text_margin:   Margins,
  pub screen_margin: Margins,
  pub screen:        Rect,
  pub border_rect:   Rect,
  pub outer_rect:    Rect,
  pub inner_rect:    Rect,
  pub border_style:  BorderStyle,
  pub margin_style:  Style,
  pub banner_style:  Style,
  pub footer_style:  Style,
}
impl ViewPort for Frame {
  fn get_view_port(&self) -> Rect {
    self.inner_rect
  }
}
impl From<Rect> for Frame {
  fn from(screen: Rect) -> Self {
    let screen_margin = Margins::default();
    let text_margin   = Margins::default();
    let border_rect   = screen_margin.get_rect(screen);
    let outer_rect    = border_rect.crop_x(1).crop_y(1);
    let inner_rect    = text_margin.get_rect(outer_rect);
    Self {
      margin_style: Style::default(),
      banner_style: Style::default(),
      footer_style: Style::default(),
      border_style: BorderStyle::default(),
      screen,
      border_rect,
      outer_rect,
      inner_rect,
      screen_margin,
      text_margin,
    }
  }
}
impl Frame {
  pub fn screen_margin(mut self, screen_margin: Margins) -> Self {
    self.screen_margin = screen_margin;
    self.border_rect = self.screen_margin.get_rect(self.screen);
    self.outer_rect  = self.border_rect.crop_x(1).crop_y(1);
    self.inner_rect  = self.text_margin.get_rect(self.outer_rect);
    self
  }

  pub fn text_margin(mut self, screen_margin: Margins) -> Self {
    self.text_margin = screen_margin;
    self.inner_rect  = self.text_margin.get_rect(self.outer_rect);
    self
  }

  pub fn banner_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.banner_style = style.into();
    self
  }

  pub fn footer_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.footer_style = style.into();
    self
  }

  pub fn margin_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.margin_style = style.into();
    self
  }

  pub fn border_style(mut self, style: BorderStyle) -> Self {
    self.border_style = style;
    self
  }

  pub fn resize(&mut self, screen: Rect) {
    self.screen      = screen;
    self.border_rect = self.screen_margin.get_rect(screen);
    self.outer_rect  = self.border_rect.crop_x(1).crop_y(1);
    self.inner_rect  = self.text_margin.get_rect(self.outer_rect);
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    self.write_frame(writer)?;
    Ok(())
  }

  pub fn write_frame<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    // border
    let (ax, ay) = self.border_rect.a();
    let (bx, by) = self.border_rect.b();
    let (cx, cy) = self.border_rect.c();
    let (dx, dy) = self.border_rect.d();
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.border_style.style)?
      .queue(MoveTo(ax, ay))?.queue(Print(self.border_style.a))?
      .queue(MoveTo(bx, by))?.queue(Print(self.border_style.b))?
      .queue(MoveTo(cx, cy))?.queue(Print(self.border_style.c))?
      .queue(MoveTo(dx, dy))?.queue(Print(self.border_style.d))?;
    for x in self.border_rect.crop_x(1).x_range() {
      writer
        .queue(MoveTo(x, ay))?.queue(Print(self.border_style.x))?
        .queue(MoveTo(x, cy))?.queue(Print(self.border_style.x))?;
    }
    for y in self.border_rect.crop_y(1).y_range() {
      writer
        .queue(MoveTo(ax, y))?.queue(Print(self.border_style.y))?
        .queue(MoveTo(bx, y))?.queue(Print(self.border_style.y))?;
    }
    // margin
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.margin_style)?;
    for x in self.outer_rect.x_range() {
      for y in self.outer_rect.y..self.inner_rect.y {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for y in self.inner_rect.y_end()..self.outer_rect.y_end() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    for y in self.inner_rect.y_range() {
      for x in self.outer_rect.x..self.inner_rect.x {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for x in self.inner_rect.x_end()..self.outer_rect.x_end() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn write_footer<W: Write>(&self, text: &str, writer: &mut W) 
    -> std::io::Result<()> 
  {
    let mut x = self.inner_rect.x_end().saturating_sub(1);
    let     y = self.border_rect.y_end().saturating_sub(1);
    writer
      .queue(MoveTo(x, y))?
      .queue(&self.border_style.style)?
      .queue(Print(self.border_style.close))?
      .queue(cursor::MoveLeft(2))?
      .queue(Print(' '))?
      .queue(&self.footer_style)?;
    x -= 2;
    for c in text.chars().rev().take(self.inner_rect.crop_x(2).w.into()) {
      writer.queue(cursor::MoveLeft(2))?.queue(Print(c))?;
      x -= 1;
    }
    writer
      .queue(cursor::MoveLeft(2))?
      .queue(Print(' '))?
      .queue(&self.border_style.style)?
      .queue(cursor::MoveLeft(2))?
      .queue(Print(self.border_style.open))?;
    x -= 2;
    for _ in self.inner_rect.x..x {
      writer
        .queue(cursor::MoveLeft(2))?
        .queue(Print(self.border_style.x))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn write_banner<W: Write>(&self, text: &str, writer: &mut W) 
    -> std::io::Result<()> 
  {
    let mut x = self.inner_rect.x;
    let     y = self.border_rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(&self.border_style.style)?
      .queue(Print(self.border_style.open))?
      .queue(Print(' '))?
      .queue(&self.banner_style)?;
    x += 2;
    for c in text.chars().take(self.inner_rect.crop_x(2).w.into()) {
      writer.queue(Print(c))?;
      x += 1;
    }
    writer
      .queue(&self.border_style.style)?
      .queue(Print(' '))?
      .queue(Print(self.border_style.close))?;
    x += 2;
    for _ in x..self.inner_rect.x_end() {
      writer.queue(Print(self.border_style.x))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
