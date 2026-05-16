// src/widget.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut, WeightedCursor,  ScreenCursor},
  rect::Rect,
  style::Style,
  text::{EditLine, StyledText, StyledTextPlane},
};
use crossterm::{
  QueueableCommand, cursor::MoveTo, style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
use std::io::{self, Write};

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
      write_unused_x: false,
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
    self.cursor.x.resize(&self.content, self.rect.x, self.rect.w);
    self.reset_state();
  }
  pub fn reset_state(&mut self) {
    self.write = true;
  }
  pub fn left(&mut self, delta: usize) -> bool {
    if self.content.backward(delta) == 0 {
      self.write = self.cursor.x.update(&self.content);
      true
    } else {false}
  }
  pub fn right(&mut self, delta: usize) -> bool {
    if self.content.forward(delta) == 0 {
      self.write = self.cursor.x.update(&self.content);
      true
    } else {false}
  }
  pub fn delete(&mut self) -> bool {
    if self.content.delete() {
      self.write_unused_x = true;
      self.cursor.x.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.content.backspace() {
      self.write_unused_x = true;
      self.cursor.x.update(&self.content);
      self.write = true;
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.content.insert(c) {
      self.cursor.x.update(&self.content);
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
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = input.iter().map(|s| StyledText::from(s.as_str()).with_style(&style));
    let rbox  = TextBox::new(rtext.collect(), &rect.cropped_north(pbox.used_rect().h))
        .write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Select(rbox),
    }
  }
  pub fn edit(prompt: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rbox  = EditBox::new(&pbox.used_rect().bottom_row()).with_style(&style);
    Dialog {
      prompt:   pbox,
      response: Response::Edit(rbox),
    }
  }
  pub fn ask(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = TextBox::new(vec![rtext], &pbox.used_rect().bottom_row()).write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Ask(rbox),
    }
  }
  pub fn ack(prompt: &str, input: &str, style: Style, rect: &Rect) -> Self {
    let ptext = StyledText::from(prompt).with_style(&style);
    let pbox  = TextBox::new(vec![ptext], &rect.cropped_south(2)).write_unused(false);
    let rtext = StyledText::from(input).with_style(&style);
    let rbox  = TextBox::new(vec![rtext], &pbox.used_rect().bottom_row()).write_unused(false);
    Dialog {
      prompt:   pbox,
      response: Response::Ack(rbox),
    }
  }
}
