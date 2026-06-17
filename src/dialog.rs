// src/dialog.rs

use crate::{
  TextBox, 
  Style, 
  TextStyle,
  ViewPort,
  Draw,
};


pub enum DlgInput {
  Ack(   TextBox),
  Ask(   TextBox),
  Select(TextBox),
  Edit(  TextBox),
}

pub struct Dialog {
  pub prompt: TextBox,
  pub input:  DlgInput,
} 

impl Dialog {
  pub fn ack<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self
  where V: ViewPort,
        S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      )
      .text(
        vec![prompt.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .text(
        vec![input.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Ack(response_box),
    }
  }

  pub fn ask<V, S>(view: V, style: S, prompt: &str, input: &str) -> Self 
  where V: ViewPort,
        S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      )
      .text(
        vec![prompt.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .text(
        vec![input.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Ask(response_box),
    }
  }

  pub fn select<V, S>(view: V, style: S, prompt: &str, input: Vec<String>) 
    -> Self 
  where V: ViewPort,
        S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      )
      .text(
        vec![prompt.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    let styles = input.iter().map(|_| TextStyle::default()).collect();
    let response_box = TextBox::from(
        view.get_view_port().crop_north(prompt_box.used_rect().h)
      )
      .text(
        input, 
        styles,
      )
      .style(style);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Select(response_box),
    }
  }

  pub fn edit<V, S>(view: V, style: S, prompt: &str, text: &str) -> Self 
  where V: ViewPort,
        S: Into<Style> + Copy
  {
    let prompt_box = TextBox::from(
        view.get_view_port().crop_south(2)
      )
      .text(
        vec![prompt.into()], 
        vec![TextStyle::default()]
      )
      .style(style);
    let response_box = TextBox::from(
        prompt_box.used_rect().bottom_row()
      )
      .text(
        vec![text.into()], 
        vec![TextStyle::default()]
      )
      .style(style)
      .as_editor();
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Edit(response_box),
    }
  }

  pub fn resize<V: ViewPort>(&mut self, rect: V) {
    self.prompt.resize(rect.get_view_port().crop_south(2));
    match &mut self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) |
      DlgInput::Edit(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      DlgInput::Select(r) => {
        r.resize(rect.get_view_port().crop_north(self.prompt.used_rect().h));
      }
    }
  }
}

impl Draw for Dialog {
  fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    self.prompt.draw(writer)?;
    match &self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) => { 
        r.draw(writer)?;
      }
      DlgInput::Edit(r) => {
        r.draw(writer)?;
        r.cursor.write(writer)?;
      }
      DlgInput::Select(r) => {
        r.draw(writer)?;
        r.cursor.write(writer)?;
      }
    }
    Ok(())
  }
}
