// src/frame.rs

use crate::{
  Rect, 
  Style, 
  Pos,
  constants::*,
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

  pub fn get_outer_rect(&self, rect: &Rect) -> Rect {
    rect
      .shift_north(self.north as i16)
      .shift_south(self.south as i16)
      .shift_east(self.east as i16)
      .shift_west(self.west as i16)
  }

  pub fn get_inner_rect(&self, rect: &Rect) -> Rect {
    rect
      .shift_north(self.north as i16 * -1)
      .shift_south(self.south as i16 * -1)
      .shift_east(self.east as i16 * -1)
      .shift_west(self.west as i16 * -1)
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

#[derive(Copy, Default, Clone)]
pub struct FrameStyle {
  pub text_margin:   Margins,
  pub screen_margin: Margins,
  pub border_style:  BorderStyle,
  pub margin_style:  Style,
  pub banner_style:  Style,
  pub footer_style:  Style,
}

impl FrameStyle {
  pub fn init() -> Self {
    Self::default()
  }

  pub fn screen_margin(mut self, screen_margin: Margins) -> Self {
    self.screen_margin = screen_margin;
    self
  }

  pub fn text_margin(mut self, screen_margin: Margins) -> Self {
    self.text_margin = screen_margin;
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

  pub fn build_from_inner(&self, rect: &Rect) -> Frame {
    let inner_rect  = rect.clone();
    let outer_rect  = self.text_margin.get_outer_rect(self.inner_rect);
    let border_rect = self.screen_margin.get_outer_rect(
      &outer_rect.shift_x(1).shift_y(1)
    );
    Frame {
      style: self.clone(),
      screen: rect.clone(),
      border_rect,
      outer_rect,
      inner_rect,
    }
  }

  pub fn build_from_outer(&self, rect: &Rect) -> Frame {
    let border_rect = self.screen_margin.get_inner_rect(rect);
    let outer_rect  = border_rect.shift_x(-1).shift_y(-1);
    let inner_rect  = self.text_margin.get_inner_rect(&outer_rect);
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
  pub style:         FrameStyle,
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
  pub fn draw_footer<W: std::io::Write>(&self, text: &str, w: &mut W) 
    -> std::io::Result<()> 
  {
    let mut x = self.inner_rect.x_end().saturating_sub(1);
    let     y = self.border_rect.y_end().saturating_sub(1);
    w
      .queue(MoveTo(x, y))?
      .queue(&self.border_style.style)?
      .queue(Print(self.border_style.close))?
      .queue(cursor::MoveLeft(2))?
      .queue(Print(' '))?
      .queue(&self.footer_style)?;
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
      .queue(&self.border_style.style)?
      .queue(cursor::MoveLeft(2))?
      .queue(Print(self.border_style.open))?;
    x -= 2;
    for _ in self.inner_rect.x()..x {
      w
        .queue(cursor::MoveLeft(2))?
        .queue(Print(self.border_style.x))?;
    }
    w.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn draw_banner<W: std::io::Write>(&self, text: &str, w: &mut W) 
    -> std::io::Result<()> 
  {
    let mut x = self.inner_rect.x();
    let     y = self.border_rect.y();
    w
      .queue(MoveTo(x, y))?
      .queue(&self.border_style.style)?
      .queue(Print(self.border_style.open))?
      .queue(Print(' '))?
      .queue(&self.banner_style)?;
    x += 2;
    for c in text
      .chars()
      .take(self.inner_rect.shift_x(-2).width().into()) 
    {
      w.queue(Print(c))?;
      x += 1;
    }
    w
      .queue(&self.border_style.style)?
      .queue(Print(' '))?
      .queue(Print(self.border_style.close))?;
    x += 2;
    for _ in x..self.inner_rect.x_end() {
      w.queue(Print(self.border_style.x))?;
    }
    w.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    // border
    let Pos(ax, ay) = self.border_rect.a();
    let Pos(bx, by) = self.border_rect.b();
    let Pos(cx, cy) = self.border_rect.c();
    let Pos(dx, dy) = self.border_rect.d();
    w
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.border_style.style)?
      .queue(MoveTo(ax, ay))?.queue(Print(self.border_style.a))?
      .queue(MoveTo(bx, by))?.queue(Print(self.border_style.b))?
      .queue(MoveTo(cx, cy))?.queue(Print(self.border_style.c))?
      .queue(MoveTo(dx, dy))?.queue(Print(self.border_style.d))?;
    for x in self.border_rect.shift_x(-1).x_range() {
      w
        .queue(MoveTo(x, ay))?.queue(Print(self.border_style.x))?
        .queue(MoveTo(x, cy))?.queue(Print(self.border_style.x))?;
    }
    for y in self.border_rect.shift_y(-1).y_range() {
      w
        .queue(MoveTo(ax, y))?.queue(Print(self.border_style.y))?
        .queue(MoveTo(bx, y))?.queue(Print(self.border_style.y))?;
    }
    // margin
    w
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.margin_style)?;
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
