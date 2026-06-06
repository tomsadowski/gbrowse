// src/widget.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut, WeightedCursor},
  view::{Rect, CursorView, ViewPort},
  style::{Style, Margins, BorderStyle},
  text::{EditLine, StyledText, StyledTextPlane},
  keys::Action,
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
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
  fn view_port(&self) -> Rect {
    self.inner_rect
  }
}
impl From<Rect> for Frame {
  fn from(screen: Rect) -> Self {
    let screen_margin = Margins::default();
    let text_margin   = Margins::default();
    let border_rect   = screen_margin.get_rect(screen);
    let outer_rect    = border_rect.clone().crop_x(1).crop_y(1);
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
  pub fn with_screen_margin(mut self, screen_margin: Margins) -> Self {
    self.screen_margin = screen_margin;
    self.border_rect = self.screen_margin.get_rect(self.screen);
    self.outer_rect  = self.border_rect.clone().crop_x(1).crop_y(1);
    self.inner_rect  = self.text_margin.get_rect(self.outer_rect);
    self
  }

  pub fn with_text_margin(mut self, screen_margin: Margins) -> Self {
    self.text_margin = screen_margin;
    self.inner_rect  = self.text_margin.get_rect(self.outer_rect);
    self
  }

  pub fn with_banner_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.banner_style = style.into();
    self
  }

  pub fn with_footer_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.footer_style = style.into();
    self
  }

  pub fn with_margin_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.margin_style = style.into();
    self
  }

  pub fn with_border_style(mut self, style: BorderStyle) -> Self {
    self.border_style = style;
    self
  }

  pub fn resize(&mut self, screen: Rect) {
    self.screen      = screen;
    self.border_rect = self.screen_margin.get_rect(screen);
    self.outer_rect  = self.border_rect.clone().crop_x(1).crop_y(1);
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
    for x in self.border_rect.cropped_x(1).x_range() {
      writer
        .queue(MoveTo(x, ay))?.queue(Print(self.border_style.x))?
        .queue(MoveTo(x, cy))?.queue(Print(self.border_style.x))?;
    }
    for y in self.border_rect.cropped_y(1).y_range() {
      writer
        .queue(MoveTo(ax, y))?.queue(Print(self.border_style.y))?
        .queue(MoveTo(bx, y))?.queue(Print(self.border_style.y))?;
    }
    // margin
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.margin_style)?;
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
    for c in text.chars().rev()
      .take(self.inner_rect.cropped_x(2).w.into()) 
    {
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
    for c in text.chars().take(self.inner_rect.cropped_x(2).w.into()) {
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

#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  pub x: CursorView,
  pub y: CursorView,
}
impl<V: ViewPort> From<&V> for ScreenCursor {
  fn from(view: &V) -> Self {
    let view = view.view_port();
    Self {
      x: CursorView::new(view.x, view.w),
      y: CursorView::new(view.y, view.h),
    }
  }
}
impl ScreenCursor {
  pub fn x_cursor(&self) -> u16 {
    self.x.view_head
  }

  pub fn y_cursor(&self) -> u16 {
    self.y.view_head
  }

  pub fn x_scroll(&self) -> usize {
    self.x.start
  }

  pub fn y_scroll(&self) -> usize {
    self.y.start
  }

  pub fn resize<X, Y>(&mut self, plane: &Y, rect: &Rect) 
  where 
    Y: UnitCursor<Unit = X> , 
    X: WeightedCursor 
  {
    self.y.resize(plane.head(), rect.y, rect.h);
    self.x.resize(
      plane.current_checked().map(|c| c.weighted_head()).unwrap_or(0), 
      rect.x, 
      rect.w
    );
  }

  pub fn update<X, Y>(&mut self, plane: &Y) -> bool 
  where 
    Y: UnitCursor<Unit = X> , 
    X: WeightedCursor
  {
    let y = self.y.update(plane.head());
    let x = self.x.update(
      plane.current_checked().map(|c| c.weighted_head()).unwrap_or(0)
    );
    x || y
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    writer
      .queue(MoveTo(self.x.cursor(), self.y.cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

#[derive(Default)]
pub struct TextBox {
  pub view:           Rect,
  pub style:          Style,
  pub content:        StyledTextPlane,
  pub cursor:         ScreenCursor,
  pub write:          bool,
  pub write_unused_x: bool,
  pub write_unused_y: bool,
}
impl<V: ViewPort> From<V> for TextBox {
  fn from(view: V) -> Self {
    Self {
      write_unused_x: true,
      write_unused_y: true,
      write:          true,
      style:          Style::default(),
      cursor:         ScreenCursor::from(&view), 
      content:        StyledTextPlane::default(),
      view:           view.view_port(),
    }
  }
}
impl TextBox {
  pub fn new<V, I, F>(view: V, input: &Vec<I>, func: F) -> Self 
  where 
    V: ViewPort,
    F: Fn(&I) -> StyledText,
  {
    Self {
      write_unused_x: true,
      write_unused_y: true,
      write:          true,
      style:          Style::default(),
      cursor:         ScreenCursor::from(&view), 
      content:        StyledTextPlane::new(&view, input, func),
      view:           view.view_port(),
    }
  }

  pub fn with_input<I, F>(mut self, input: &Vec<I>, func: F) -> Self 
  where 
    F: Fn(&I) -> StyledText,
  {
    self.content = StyledTextPlane::new(&self.view, input, func);
    self
  }

  pub fn set_input<I, F>(&mut self, input: &Vec<I>, func: F)
  where 
    F: Fn(&I) -> StyledText,
  {
    self.content = StyledTextPlane::new(&self.view, input, func);
  }

  pub fn with_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.style = style.into();
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
    self
      .write_unused_x(write)
      .write_unused_y(write)
  }

  pub fn get_source_idx(&self) -> usize {
    self.content.get_source_idx()
  }

  pub fn get_source(&self) -> String {
    self.content.get_source()
  }

  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.content.units().len()) {
      self.view.clone().cap_height(h)
    } else {
      self.view.clone()
    }
  }

  pub fn reset_state(&mut self) {
    self.write = true;
  }

  pub fn restyle<I, F>(&mut self, input: &Vec<I>, func: F)
  where F: Fn(&I) -> StyledText,
  {
    self.content.restyle(self.view, input, func);
    self.reset_state();
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    self.view = view.view_port();
    self.content.resize(self.view.w);
    self.cursor.resize(&self.content, &self.view);
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

  pub fn update(&mut self, action: &Action) {
    match action {
      Action::PageDown  => {self.down(usize::from(self.view.h));}
      Action::PageUp    => {self.up(usize::from(self.view.h));}
      Action::Bottom    => {self.down(self.content.units().len());}
      Action::Top       => {self.up(self.content.units().len());}
      Action::MoveDown  => {self.down(1);}
      Action::MoveUp    => {self.up(1);}
      Action::MoveLeft  => {self.left(1);}
      Action::MoveRight => {self.right(1);}
      _ => {}
    }
  }

  pub fn empty<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for y in self.view.y_range() {
      for x in self.view.x_range() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }

  pub fn write_all<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    let mut x = self.view.x;
    let mut y = self.view.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for line in self.content
      .view_units(self.cursor.y_scroll(), self.view.h.into()) 
    {
      writer.queue(&self.content.source[line.idx].style)?;
      for c in line
        .view_weighted(self.cursor.x_scroll(), self.view.w.into()) 
      {
        writer.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      if self.write_unused_x {
        writer
          .queue(SetAttribute(Attribute::Reset))?
          .queue(&self.style)?;
        for _ in x..self.view.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.view.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    if self.write_unused_y {
      writer
        .queue(SetAttribute(Attribute::Reset))?
        .queue(&self.style)?;
      for _ in y..self.view.y_end() {
        for _ in self.view.x_range() {
          writer.queue(Print(' '))?;
        }
        x = self.view.x;
        y += 1;
        writer.queue(MoveTo(x, y))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}

#[derive(Default)]
pub struct EditBox {
  pub style:          Style,
  pub write:          bool,
  pub rect:           Rect,
  pub content:        EditLine,
  pub cursor:         ScreenCursor,
  pub write_unused_x: bool,
}
impl<V: ViewPort> From<V> for EditBox {
  fn from(view: V) -> Self {
    let view = view.view_port().top_row();
    Self {
      write_unused_x: true,
      write:          true,
      style:          Style::default(),
      cursor:         ScreenCursor::from(&view), 
      content:        EditLine::default(), 
      rect: view,
    }
  }
}
impl EditBox {
  pub fn with_text(mut self, text: &str) -> Self {
    self.content = EditLine::from(text);
    self.cursor.x.update(self.content.weighted_head());
    self
  }

  pub fn with_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    self
  }

  pub fn write_unused_x(mut self, write: bool) -> Self {
    self.write_unused_x = write;
    self
  }

  pub fn resize<V: ViewPort>(&mut self, rect: V) {
    self.rect = rect.view_port().top_row();
    self.cursor.x.resize(
      self.content.weighted_head(), 
      self.rect.x, 
      self.rect.w
      );
    self.reset_state();
  }

  pub fn reset_state(&mut self) {
    self.write = true;
  }

  pub fn left(&mut self, delta: usize) -> bool {
    if self.content.backward(delta) == 0 {
      self.write = self.cursor.x.update(self.content.weighted_head());
      true
    } else {false}
  }

  pub fn right(&mut self, delta: usize) -> bool {
    if self.content.forward(delta) == 0 {
      self.write = self.cursor.x.update(self.content.weighted_head());
      true
    } else {false}
  }

  pub fn delete(&mut self) -> bool {
    if self.content.delete() {
      self.cursor.x.update(self.content.weighted_head());
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.content.backspace() {
      self.cursor.x.update(self.content.weighted_head());
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, c: char) -> bool {
    if self.content.insert(c) {
      self.cursor.x.update(self.content.weighted_head());
      self.write = true;
      true
    } else {false}
  }

  pub fn update(&mut self, action: &Action) {
    match action {
      Action::Backspace => {self.backspace();}
      Action::Delete    => {self.delete();}
      Action::Insert(c) => {self.insert(*c);}
      Action::MoveLeft  => {self.left(1);}
      Action::MoveRight => {self.right(1);}
      _ => {}
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }

  pub fn write_all<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    let mut x = self.rect.x;
    let     y = self.rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    // render chars
    for c in self.content
      .view_weighted(self.cursor.x_scroll(), self.rect.w.into()) 
    {
      writer.queue(Print(c))?;
      x += u16::try_from(c.width().unwrap_or(0)).unwrap();
    }
    writer.queue(MoveTo(x, y))?;
    // render page space
    if self.write_unused_x {
      for _ in x..self.rect.x_end() {
        writer.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
