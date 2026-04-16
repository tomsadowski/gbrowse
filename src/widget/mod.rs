// src/widget/mod.rs

mod frame;
mod textbox;
mod editbox;
mod dynamo;

pub use self::frame::Frame;
pub use self::textbox::TextBox;
pub use self::editbox::EditBox;
pub use self::dynamo::Dynamo;

use crate::{
  common as c,
  screen::{Rect, PlaneView},
  text::{Style, StyledText}, 
};
use crossterm::{
  QueueableCommand, 
  style::{SetAttribute, Attribute, Color},
  cursor::{self, MoveTo},
};
use std::{
  ops::Range,
  io::{self, Write}
};

pub fn write_reset<W: Write>(writer: &mut W) -> io::Result<()> {
  writer.queue(SetAttribute(Attribute::Reset))?;
  Ok(())
}
pub fn cursor_hide<W: Write>(writer: &mut W) -> io::Result<()> {
  writer.queue(cursor::Hide)?;
  Ok(())
}
pub trait PlaneWidget {
  fn pos(&self) -> (u16, u16);
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()>;

  fn write_cursor<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let (x, y) = self.pos();
    writer.queue(MoveTo(x, y))?.queue(cursor::Show)?;
    Ok(())
  }
}
impl PlaneWidget for TextBox {
  fn pos(&self) -> (u16, u16) {
    (self.pos.x_cursor(), self.pos.y_cursor())
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if let Some(ChangeType::Scroll) = self.change {
      self.write_all(writer)?;
    }
    Ok(())
  }
}
impl PlaneWidget for Dynamo {
  fn pos(&self) -> (u16, u16) {
    (self.pos.x_cursor(), self.pos.y_cursor())
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if let Some(ChangeType::Scroll) = self.change {
      self.write_all(writer)?;
    }
    Ok(())
  }
}
impl PlaneWidget for EditBox {
  fn pos(&self) -> (u16, u16) {
    (self.pos.cursor(), self.rect.y)
  }
  fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if let Some(ChangeType::Scroll) = self.change {
      self.write_all(writer)?;
    }
    Ok(())
  }
}

pub enum Response {
  Ack(Dynamo),
  Ask(Dynamo),
  Text(EditBox),
  Select(Dynamo),
}
pub struct Dialog {
  pub prompt:   Dynamo,
  pub response: Response,
} 
impl Dialog {
  pub fn select(prompt: &str, instruction: Vec<String>, style: Style, rect: &Rect) -> Self {
    let prompt_text   = StyledText::from(prompt).with_style(&style);
    let prompt_box    = Dynamo::new(vec![prompt_text], &rect.clone().crop_south(2));
    let response_text = instruction.iter().map(|s| StyledText::from(s.as_str()).with_style(&style));
    let response_box  = 
      Dynamo::new(response_text.collect(), &rect.clone().crop_north(prompt_box.used.h));
    Dialog {
      prompt:   prompt_box,
      response: Response::Select(response_box),
    }
  }
  pub fn text(prompt: &str, style: Style, rect: &Rect) -> Self {
    let prompt_text  = StyledText::from(prompt).with_style(&style);
    let prompt_box   = Dynamo::new(vec![prompt_text], &rect.clone().crop_south(2));
    let response_box = EditBox::new(&prompt_box.used.bottom_row()).with_style(&style);
    Dialog {
      prompt:   prompt_box,
      response: Response::Text(response_box),
    }
  }
  pub fn ask(prompt: &str, instruction: &str, style: Style, rect: &Rect) -> Self {
    let prompt_text   = StyledText::from(prompt).with_style(&style);
    let prompt_box    = Dynamo::new(vec![prompt_text], &rect.clone().crop_south(2));
    let response_text = StyledText::from(instruction).with_style(&style);
    let response_box  = Dynamo::new(vec![response_text], &prompt_box.used.bottom_row());
    Dialog {
      prompt:   prompt_box,
      response: Response::Ask(response_box),
    }
  }
  pub fn ack(prompt: &str, instruction: &str, style: Style, rect: &Rect) -> Self {
    let prompt_text   = StyledText::from(prompt).with_style(&style);
    let prompt_box    = Dynamo::new(vec![prompt_text], &rect.clone().crop_south(2));
    let response_text = StyledText::from(instruction).with_style(&style);
    let response_box  = Dynamo::new(vec![response_text], &prompt_box.used.bottom_row());
    Dialog {
      prompt:   prompt_box,
      response: Response::Ack(response_box),
    }
  }
}

#[derive(Clone, Debug)]
pub enum ChangeType {
  Cursor, Scroll,
}
#[derive(Debug, Clone)]
pub struct MarginSpec {
  pub north: u16,
  pub south: u16,
  pub east:  u16,
  pub west:  u16,
}
impl Default for MarginSpec {
  fn default() -> Self {
    Self {north: 0, south: 0, east: 0, west: 0}
  }
}
impl MarginSpec {
  pub fn get_rect(&self, screen: &Rect) -> Rect {
    screen.clone()
      .crop_north(self.north).crop_south(self.south)
      .crop_east(self.east).crop_west(self.west)
  }
}
#[derive(Debug, Clone)]
pub struct BorderSpec {
  pub style: Style,
  pub a:     char,
  pub b:     char,
  pub c:     char,
  pub d:     char,
  pub open:  char,
  pub close: char,
}
impl Default for BorderSpec {
  fn default() -> Self {
    Self {
      style: Style::default(),
      a:     c::A_SQR,
      b:     c::B_SQR,
      c:     c::C_SQR,
      d:     c::D_SQR,
      open:  c::OPEN_SQR,
      close: c::CLOSE_SQR,
    }
  }
}
