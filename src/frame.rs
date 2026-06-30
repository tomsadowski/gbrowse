// src/frame.rs

use crate::{
  Rect, 
  Style,
};


#[derive(Copy, Debug, Clone, Default)]
pub struct Margins {
  pub north: u16,
  pub south: u16,
  pub east:  u16,
  pub west:  u16,
}
impl Margins {
  pub fn symmetric(m: u16) -> Self {
    Self {
      north: m, 
      south: m, 
      east:  m, 
      west:  m,
    }
  }
  pub fn get_from_inner(&self, rect: &Rect) -> Rect {
    rect
      .shift_x(self.north as i16)
      .shift_y(self.south as i16)
  }
  pub fn get_from_outer(&self, rect: &Rect) -> Rect {
    rect
      .shift_x(self.north as i16 * -1)
      .shift_y(self.south as i16 * -1)
  }
}

#[derive(Copy, Debug, Clone)]
pub struct BorderStyle {
  pub style: Style,
  pub x:     char,
  pub y:     char,
  pub a:     char,
  pub b:     char,
  pub c:     char,
  pub d:     char,
  pub open:  char,
  pub close: char,
}
impl Default for BorderStyle {
  fn default() -> Self {
    use crate::constants::*;
    Self {
      style: Style::default(),
      x:     X_LINE,
      y:     Y_LINE,
      a:     A_SQR,
      b:     B_SQR,
      c:     C_SQR,
      d:     D_SQR,
      open:  ' ',
      close: ' ',
    }
  }
}
impl BorderStyle {
  pub fn get_from_inner(&self, rect: &Rect) -> Rect {
    rect.shift_x(1).shift_y(1)
  }
  pub fn get_from_outer(&self, rect: &Rect) -> Rect {
    rect.shift_x(-1).shift_y(-1)
  }
}

#[derive(Copy, Default, Clone)]
pub struct FrameParams {
  pub text_margin:   Margins,
  pub screen_margin: Margins,
  pub border:        Option<BorderStyle>,
  pub margin:        Style,
  pub banner:        Style,
  pub footer:        Style,
}
impl FrameParams {
  pub fn init() -> Self { Self::default() }
  pub fn screen_margin(mut self, screen_margin: Margins) -> Self {
    self.screen_margin = screen_margin;
    self
  }
  pub fn text_margin(mut self, screen_margin: Margins) -> Self {
    self.text_margin = screen_margin;
    self
  }
  pub fn banner_style(mut self, style: impl Into<Style> + Copy) -> Self {
    self.banner = style.into();
    self
  }
  pub fn footer_style(mut self, style: impl Into<Style> + Copy) -> Self {
    self.footer = style.into();
    self
  }
  pub fn margin_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.margin = style.into();
    self
  }
  pub fn border_style(mut self, style: Option<BorderStyle>) -> Self {
    self.border = style;
    self
  }
  pub fn build_from_inner(&self, rect: &Rect) -> Frame {
    let inner_rect  = rect.clone();
    let outer_rect  = self.text_margin.get_from_inner(&inner_rect);
    let border_rect = 
      if let None = self.border { outer_rect } 
      else { outer_rect.shift_x(1).shift_y(1) };
    Frame {
      style: self.clone(),
      screen: rect.clone(),
      border_rect,
      outer_rect,
      inner_rect,
    }
  }
  pub fn build_from_outer(&self, rect: &Rect) -> Frame {
    let border_rect = self.screen_margin.get_from_outer(rect);
    let outer_rect = 
      if let None = self.border { border_rect } 
      else { border_rect.shift_x(-1).shift_y(-1) };
    let inner_rect  = self.text_margin.get_from_outer(&outer_rect);
    Frame {
      style: self.clone(),
      screen: rect.clone(),
      border_rect,
      outer_rect,
      inner_rect,
    }
  }
}

#[derive(Copy, Default, Clone)]
pub struct Frame {
  pub style:         FrameParams,
  pub screen:        Rect,
  pub border_rect:   Rect,
  pub outer_rect:    Rect,
  pub inner_rect:    Rect,
}
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
impl Frame {
  pub fn draw_footer(&self, text: &str, w: &mut impl std::io::Write) 
  -> std::io::Result<()> {
    if let Some(border) = self.style.border {
      let mut x = self.inner_rect.x_end().saturating_sub(1);
      let     y = self.border_rect.y_end().saturating_sub(1);
      w
        .queue(MoveTo(x, y))?
        .queue(&border.style)?
        .queue(Print(border.close))?
        .queue(cursor::MoveLeft(2))?
        .queue(Print(' '))?
        .queue(&self.style.footer)?;
      x -= 2;
      for c in text
        .chars()
        .rev()
        .take(self.inner_rect.shift_x(-2).width().into()) 
      {
        w.queue(cursor::MoveLeft(2))?.queue(Print(c))?;
        x -= 1;
      }
      w
        .queue(cursor::MoveLeft(2))?
        .queue(Print(' '))?
        .queue(&border.style)?
        .queue(cursor::MoveLeft(2))?
        .queue(Print(border.open))?;
      x -= 2;
      for _ in self.inner_rect.x()..x {
        w
          .queue(cursor::MoveLeft(2))?
          .queue(Print(border.x))?;
      }
      w.queue(SetAttribute(Attribute::Reset))?;
    }
    Ok(())
  }
  pub fn draw_banner(&self, text: &str, w: &mut impl std::io::Write) 
  -> std::io::Result<()> {
    if let Some(border) = self.style.border {
      let mut x = self.inner_rect.x();
      let     y = self.border_rect.y();
      w
        .queue(MoveTo(x, y))?
        .queue(&border.style)?
        .queue(Print(border.open))?
        .queue(Print(' '))?
        .queue(&self.style.banner)?;
      x += 2;
      for c in text
        .chars()
        .take(self.inner_rect.shift_x(-2).width().into()) 
      {
        w.queue(Print(c))?;
        x += 1;
      }
      w
        .queue(&border.style)?
        .queue(Print(' '))?
        .queue(Print(border.close))?;
      x += 2;
      for _ in x..self.inner_rect.x_end() {
        w.queue(Print(border.x))?;
      }
      w.queue(SetAttribute(Attribute::Reset))?;
    }
    Ok(())
  }
  pub fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    // border
    if let Some(border) = self.style.border {
      let (ax, ay) = self.border_rect.a().into();
      let (bx, by) = self.border_rect.b().into();
      let (cx, cy) = self.border_rect.c().into();
      let (dx, dy) = self.border_rect.d().into();
      w
        .queue(SetAttribute(Attribute::Reset))?
        .queue(&border.style)?
        .queue(MoveTo(ax, ay))?.queue(Print(border.a))?
        .queue(MoveTo(bx, by))?.queue(Print(border.b))?
        .queue(MoveTo(cx, cy))?.queue(Print(border.c))?
        .queue(MoveTo(dx, dy))?.queue(Print(border.d))?;
      for x in self.border_rect.shift_x(-1).x_range() {
        w
          .queue(MoveTo(x, ay))?.queue(Print(border.x))?
          .queue(MoveTo(x, cy))?.queue(Print(border.x))?;
      }
      for y in self.border_rect.shift_y(-1).y_range() {
        w
          .queue(MoveTo(ax, y))?.queue(Print(border.y))?
          .queue(MoveTo(bx, y))?.queue(Print(border.y))?;
      }
    }
    // margin
    w
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style.margin)?;
    for x in self.outer_rect.x_range() {
      for y in self.outer_rect.y()..self.inner_rect.y() {
        w.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for y in self.inner_rect.y_end()..self.outer_rect.y_end() {
        w.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    for y in self.inner_rect.y_range() {
      for x in self.outer_rect.x()..self.inner_rect.x() {
        w.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
      for x in self.inner_rect.x_end()..self.outer_rect.x_end() {
        w.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    w.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
