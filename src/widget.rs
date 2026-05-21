// src/widget.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut, WeightedCursor},
  view::{Rect, CursorView},
  style::{Style, MarginSpec, BorderSpec},
  text::{EditLine, StyledText, StyledTextPlane},
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
use std::io::{self, Write};


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
      .queue(SetAttribute(Attribute::Reset))?.queue(&self.border_spec.style)?
      .queue(MoveTo(ax, ay))?.queue(Print(self.border_spec.a))?
      .queue(MoveTo(bx, by))?.queue(Print(self.border_spec.b))?
      .queue(MoveTo(cx, cy))?.queue(Print(self.border_spec.c))?
      .queue(MoveTo(dx, dy))?.queue(Print(self.border_spec.d))?;
    for x in self.border_rect.cropped_x(1).x_range() {
      writer
        .queue(MoveTo(x, ay))?.queue(Print(self.border_spec.x))?
        .queue(MoveTo(x, cy))?.queue(Print(self.border_spec.x))?;
    }
    for y in self.border_rect.cropped_y(1).y_range() {
      writer
        .queue(MoveTo(ax, y))?.queue(Print(self.border_spec.y))?
        .queue(MoveTo(bx, y))?.queue(Print(self.border_spec.y))?;
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
      .queue(cursor::MoveLeft(2))?.queue(Print(' '))?
      .queue(&self.banner_style)?;
    x -= 2;
    for c in text.chars().rev().take(self.inner_rect.cropped_x(2).w.into()) {
      writer.queue(cursor::MoveLeft(2))?.queue(Print(c))?;
      x -= 1;
    }
    writer
      .queue(cursor::MoveLeft(2))?.queue(Print(' '))?
      .queue(&self.border_spec.style)?
      .queue(cursor::MoveLeft(2))?.queue(Print(self.border_spec.open))?;
    x -= 2;
    for _ in self.inner_rect.x..x {
      writer.queue(cursor::MoveLeft(2))?.queue(Print(self.border_spec.x))?;
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
      .queue(Print(' '))?.queue(Print(self.border_spec.close))?;
    x += 2;
    for _ in x..self.inner_rect.x_end() {
      writer.queue(Print(self.border_spec.x))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
#[derive(Clone, Debug, Default)]
pub struct ScreenCursor {
  pub x: CursorView,
  pub y: CursorView,
}
impl ScreenCursor {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: CursorView::new(rect.x, rect.w),
      y: CursorView::new(rect.y, rect.h),
    }
  }
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
  where Y: UnitCursor<Unit = X> , X: WeightedCursor 
  {
    self.y.resize(plane.head(), rect.y, rect.h);
    self.x.resize(plane.current().weighted_head(), rect.x, rect.w);
  }
  pub fn update<X, Y>(&mut self, plane: &Y) -> bool 
  where Y: UnitCursor<Unit = X> , X: WeightedCursor
  {
    let y = self.y.update(plane.head());
    let x = self.x.update(plane.current().weighted_head());
    x || y
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer
      .queue(MoveTo(self.x.cursor(), self.y.cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

#[derive(Default)]
pub struct TextBox {
  pub rect:           Rect,
  pub style:          Style,
  pub content:        StyledTextPlane,
  pub cursor:         ScreenCursor,
  pub write:          bool,
  pub write_unused_x: bool,
  pub write_unused_y: bool,
}
impl TextBox {
  pub fn new(text: Vec<StyledText>, rect: &Rect) -> Self {
    let content = StyledTextPlane::new(text, rect.w);
    let pos     = ScreenCursor::new(&rect);
    Self {
      write_unused_x: true,
      write_unused_y: true,
      style:          Style::default(),
      write:          true,
      rect:           rect.clone(),
      cursor:         pos, 
      content,
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
  pub fn get_source_idx(&self) -> usize {
    self.content.current().idx
  }
  pub fn get_source(&self) -> String {
    self.content.get_source()
  }
  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.content.units().len()) {
      self.rect.clone().cap_height(h)
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
    self.cursor.resize(&self.content, &rect);
    self.reset_state();
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.content.resize(rect.w);
    self.cursor.resize(&self.content, &rect);
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
  pub fn clear<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    for y in self.rect.y_range() {
      for x in self.rect.x_range() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let mut y = self.rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    for line in self.content.view_units(self.cursor.y_scroll(), self.rect.h.into()) {
      writer.queue(&self.content.source[line.idx].style)?;
      for c in line.view_weighted(self.cursor.x_scroll(), self.rect.w.into()) {
        writer.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      if self.write_unused_x {
        writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
        for _ in x..self.rect.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.rect.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    if self.write_unused_y {
      writer.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
      for _ in y..self.rect.y_end() {
        for _ in self.rect.x_range() {
          writer.queue(Print(' '))?;
        }
        x = self.rect.x;
        y += 1;
        writer.queue(MoveTo(x, y))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}

// coordinate Page and PlaneView
#[derive(Default)]
pub struct EditBox {
  pub style:          Style,
  pub write:          bool,
  pub rect:           Rect,
  pub content:        EditLine,
  pub cursor:         ScreenCursor,
  pub write_unused_x: bool,
}
impl EditBox {
  pub fn new(rect: &Rect) -> Self {
    let content = EditLine::from("");
    let rect    = rect.top_row();
    let pos     = ScreenCursor::new(&rect);
    Self {
      rect:           rect.clone(),
      style:          Style::default(),
      write_unused_x: true,
      write:          true,
      cursor: pos, 
      content, 
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
  pub fn resize(&mut self, rect: &Rect) {
    self.rect = rect.top_row();
    self.cursor.x.resize(self.content.weighted_head(), self.rect.x, self.rect.w);
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
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }
  pub fn write_all<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let mut x = self.rect.x;
    let     y = self.rect.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    // render chars
    for c in self.content.view_weighted(self.cursor.x_scroll(), self.rect.w.into()) {
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

pub enum Response {
  Ack(TextBox),
  Ask(TextBox),
  Edit(EditBox),
  Select(TextBox),
}

pub struct Dialog {
  pub prompt:   TextBox,
  pub response: Response,
} 
impl Dialog {
  pub fn resize(&mut self, rect: &Rect) {
    self.prompt.resize(&rect.cropped_south(2));
    match &mut self.response {
      Response::Ack(r)    => r.resize(&self.prompt.used_rect().bottom_row()),
      Response::Ask(r)    => r.resize(&self.prompt.used_rect().bottom_row()),
      Response::Edit(r)   => r.resize(&self.prompt.used_rect().bottom_row()),
      Response::Select(r) => r.resize(&rect.cropped_north(self.prompt.used_rect().h)),
    }
  }
  pub fn select(prompt: &str, input: Vec<String>, style: Style, rect: &Rect) -> Self {
    let prompt_box = TextBox::new(
        vec![StyledText::from(prompt).with_style(&style)], 
        &rect.cropped_south(2)
      )
      .write_unused_y(false)
      .with_style(&style);
    let response_box  = TextBox::new(
        input.iter().map(|s| StyledText::from(s.as_str()).with_style(&style)).collect(), 
        &rect.cropped_north(prompt_box.used_rect().h)
      )
      .write_unused_y(false)
      .with_style(&style);
    Dialog {
      prompt:   prompt_box,
      response: Response::Select(response_box),
    }
  }
  pub fn edit(prompt: &str, style: Style, rect: &Rect) -> Self {
    let prompt_box = TextBox::new(
        vec![StyledText::from(prompt).with_style(&style)],
        &rect.cropped_south(2)
      )
      .write_unused_y(false)
      .with_style(&style);
    let response_box  = EditBox::new(
        &prompt_box.used_rect().bottom_row()
      )
      .with_style(&style);
    Dialog {
      prompt:   prompt_box,
      response: Response::Edit(response_box),
    }
  }
  pub fn ask(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let prompt_box = TextBox::new(
        vec![StyledText::from(prompt).with_style(&style)], 
        &rect.cropped_south(2)
      )
      .write_unused_y(false)
      .with_style(&style);
    let response_box = TextBox::new(
        vec![StyledText::from(input).with_style(&style)], 
        &prompt_box.used_rect().bottom_row()
      )
      .write_unused_y(false)
      .with_style(&style);
    Dialog {
      prompt:   prompt_box,
      response: Response::Ask(response_box),
    }
  }
  pub fn ack(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let prompt_box = TextBox::new(
        vec![StyledText::from(prompt).with_style(&style)], 
        &rect.cropped_south(2)
      )
      .write_unused_y(false)
      .with_style(&style);
    let response_box = TextBox::new(
        vec![StyledText::from(input).with_style(&style)], 
        &prompt_box.used_rect().bottom_row()
      )
      .write_unused_y(false)
      .with_style(&style);
    Dialog {
      prompt:   prompt_box,
      response: Response::Ack(response_box),
    }
  }
}
