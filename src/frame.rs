// src/frame.rs

use crate::{
  Rect, 
  Style, 
  Margins, 
  BorderStyle,
  Pos,
};


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

impl crate::GetRect for Frame {
  fn get_rect(&self) -> Rect { self.inner_rect }
}

impl From<Rect> for Frame {
  fn from(screen: Rect) -> Self {
    let screen_margin = Margins::default();
    let text_margin   = Margins::default();
    let border_rect   = screen_margin.get_inner(screen);
    let outer_rect    = border_rect.shift_x(-1).shift_y(-1);
    let inner_rect    = text_margin.get_inner(outer_rect);
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
    self.border_rect = self.screen_margin.get_inner(self.screen);
    self.outer_rect  = self.border_rect.shift_x(-1).shift_y(-1);
    self.inner_rect  = self.text_margin.get_inner(self.outer_rect);
    self
  }

  pub fn text_margin(mut self, screen_margin: Margins) -> Self {
    self.text_margin = screen_margin;
    self.inner_rect  = self.text_margin.get_inner(self.outer_rect);
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

  pub fn resize_inner(&mut self, inner_rect: Rect) {
    self.inner_rect  = inner_rect;
    self.outer_rect  = self.text_margin.get_outer(self.inner_rect);
    self.border_rect = self.screen_margin.get_outer(
      self.outer_rect.shift_x(1).shift_y(1)
    );
  }

  pub fn resize(&mut self, screen: Rect) {
    self.screen      = screen;
    self.border_rect = self.screen_margin.get_inner(screen);
    self.outer_rect  = self.border_rect.shift_x(-1).shift_y(-1);
    self.inner_rect  = self.text_margin.get_inner(self.outer_rect);
  }

  pub fn reset(&mut self, inner_rect: Rect) {
    self.resize(self.screen);
  }

  pub fn draw_footer<W: std::io::Write>(&self, text: &str, w: &mut W) 
    -> std::io::Result<()> 
  {
    use crossterm::{
      QueueableCommand, 
      cursor::{self, MoveTo}, 
      style::{Print, SetAttribute, Attribute},
    };
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
    use crossterm::{
      QueueableCommand, 
      cursor::MoveTo, 
      style::{Print, SetAttribute, Attribute},
    };
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
    use crossterm::{
      QueueableCommand, 
      cursor::MoveTo, 
      style::{Print, SetAttribute, Attribute},
    };
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
