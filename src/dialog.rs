// src/dialog.rs

use crate::{
  TextLine, 
  EditLine, 
  TextBox, 
  StyledText, 
  Style, 
  ViewPort,
};
use std::io::Write;


pub enum DlgInput {
  Ack(   TextBox<TextLine>),
  Ask(   TextBox<TextLine>),
  Select(TextBox<TextLine>),
  Edit(  TextBox<EditLine>),
}

pub struct Dialog {
  pub prompt: TextBox<TextLine>,
  pub input:  DlgInput,
} 

impl Dialog {
  pub fn ack<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      ).reference(
        &vec![prompt], |p| StyledText::from(*p).style(style)
      ).style(style).write_unused_y(false);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      ).reference(
        &vec![input], |i| StyledText::from(*i).style(style)
      ).style(style).write_unused_y(false);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Ack(response_box),
    }
  }

  pub fn ask<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      ).reference(
        &vec![prompt], |p| StyledText::from(*p).style(style)
      ).style(style).write_unused_y(false);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      ).reference(
        &vec![input], |i| StyledText::from(*i).style(style)
      ).style(style).write_unused_y(false);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Ask(response_box),
    }
  }

  pub fn select<V, S>(view: V, style: S, prompt: &str, input: Vec<String>) 
    -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      ).reference(
        &vec![prompt], |p| StyledText::from(*p).style(style)
      ).style(style).write_unused_y(false);
    let response_box = TextBox::from(
        view.get_view_port().crop_north(prompt_box.used_rect().h)
      ).reference(
        &input, |s| StyledText::from(s.as_str()).style(style)
      ).style(style).write_unused_y(false);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Select(response_box),
    }
  }

  pub fn edit<V, S>(view: V, style: S, prompt: &str, text: &str) -> Self 
  where 
    V: ViewPort,
    S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      ).reference(
        &vec![prompt], |p| StyledText::from(*p).style(style)
      ).style(style).write_unused_y(false);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      ).reference(
        &vec![text], |p| StyledText::from(*p).style(style)
      ).style(style);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Edit(response_box),
    }
  }

  pub fn resize<V: ViewPort>(&mut self, rect: V) {
    self.prompt.resize(rect.get_view_port().crop_south(2));
    match &mut self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      DlgInput::Edit(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      DlgInput::Select(r) => {
        r.resize(rect.get_view_port().crop_north(self.prompt.used_rect().h));
      }
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    self.prompt.write(writer, 0)?;
    match &self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) => { 
        r.write(writer, 0)?;
      }
      DlgInput::Edit(r) => {
        r.write(writer, 0)?;
        r.cursor.write(writer)?;
      }
      DlgInput::Select(r) => {
        r.write(writer, 0)?;
        r.cursor.write(writer)?;
      }
    }
    Ok(())
  }
}
