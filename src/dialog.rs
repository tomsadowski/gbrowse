// src/dialog.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut, WeightedCursor},
  widget::{TextBox, EditBox},
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


pub enum Response {
  Ack(TextBox),
  Ask(TextBox),
  Edit(EditBox),
  Select(TextBox),
}

pub struct Dlg {
  pub prompt:   TextBox,
  pub response: Response,
} 
impl Dlg {
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
    Dlg {
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
    Dlg {
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
    Dlg {
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
    Dlg {
      prompt:   prompt_box,
      response: Response::Ack(response_box),
    }
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    self.prompt.write(writer)?;
    match &self.response {
      Response::Ack(r) | Response::Ask(r) => 
        r.write(writer)?,
      Response::Edit(r) => {
        r.write(writer)?;
        r.cursor.write(writer)?;
      }
      Response::Select(r) => {
        r.write(writer)?;
        r.cursor.write(writer)?;
      }
    }
    Ok(())
  }
}
