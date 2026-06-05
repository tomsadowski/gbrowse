// src/dialog.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut, WeightedCursor},
  widget::{TextBox, EditBox},
  view::{Rect, CursorView, ViewPort},
  style::{Style, Margins, BorderStyle},
  text::{EditLine, StyledText, StyledTextPlane},
};
use crossterm::QueueableCommand;
use unicode_width::UnicodeWidthChar;
use std::io::Write;


pub enum Input {
  Ack(TextBox),
  Ask(TextBox),
  Select(TextBox),
  Edit(EditBox),
}

pub struct Dialog {
  pub prompt:   TextBox,
  pub input: Input,
} 
impl Dialog {
  pub fn ack<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.view_port().cropped_south(2)
      )
      .with_input(
        &vec![prompt], |p| StyledText::from(*p).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .with_input(
        &vec![input], |i| StyledText::from(*i).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    Dialog {
      prompt:   prompt_box,
      input: Input::Ack(response_box),
    }
  }

  pub fn ask<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.view_port().cropped_south(2)
      )
      .with_input(
        &vec![prompt], |p| StyledText::from(*p).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .with_input(
        &vec![input], |i| StyledText::from(*i).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    Dialog {
      prompt:   prompt_box,
      input: Input::Ask(response_box),
    }
  }

  pub fn select<V, S>(view: V, style: S, prompt: &str, input: Vec<String>) 
    -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.view_port().cropped_south(2)
      )
      .with_input(
        &vec![prompt], |p| StyledText::from(*p).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    let response_box = TextBox::from(
        view.view_port().cropped_north(prompt_box.used_rect().h)
      )
      .with_input(
        &input, |s| StyledText::from(s.as_str()).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    Dialog {
      prompt:   prompt_box,
      input: Input::Select(response_box),
    }
  }

  pub fn edit<V, S>(view: V, style: S, prompt: &str, text: &str) -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.view_port().cropped_south(2)
      )
      .with_input(
        &vec![prompt], |p| StyledText::from(*p).with_style(style)
      )
      .with_style(style)
      .write_unused_y(false);
    let response_box = EditBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .with_text(text)
      .with_style(style);
    Dialog {
      prompt:   prompt_box,
      input: Input::Edit(response_box),
    }
  }

  pub fn resize<V: ViewPort>(&mut self, rect: V) {
    self.prompt.resize(rect.view_port().cropped_south(2));
    match &mut self.input {
      Input::Ack(r) | 
      Input::Ask(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      Input::Edit(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      Input::Select(r) => {
        r.resize(rect.view_port().cropped_north(self.prompt.used_rect().h));
      }
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    self.prompt.write(writer)?;
    match &self.input {
      Input::Ack(r) | 
      Input::Ask(r) => { 
        r.write(writer)?;
      }
      Input::Edit(r) => {
        r.write(writer)?;
        r.cursor.write(writer)?;
      }
      Input::Select(r) => {
        r.write(writer)?;
        r.cursor.write(writer)?;
      }
    }
    Ok(())
  }
}
