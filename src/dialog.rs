// src/dialog.rs

use crate::{
  TextBox, 
  Style, 
  ViewPort,
};


pub enum DlgInput {
  Ack   (TextBox),
  Ask   (TextBox),
  Select(TextBox),
  Edit  (TextBox),
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
        view.get_view_port()
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = TextBox::from(
        view.get_view_port()
      )
      .text(
        vec![input.into()], 
        vec![]
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
        view.get_view_port()
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = TextBox::from(
        view.get_view_port()
      )
      .text(
        vec![input.into()], 
        vec![]
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
        view.get_view_port()
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = TextBox::from(
        view.get_view_port()
      )
      .text(
        input, 
        vec![]
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
        view.get_view_port()
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = TextBox::from(
        view.get_view_port()
      )
      .text(
        vec![text.into()], 
        vec![]
      )
      .style(style)
      .editor();
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Edit(response_box),
    }
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    self.prompt.resize(view.get_view_port());
    match &mut self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) |
      DlgInput::Edit(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      DlgInput::Select(r) => {
        r.resize(view.get_view_port().crop_north(self.prompt.used_rect().h));
      }
    }
  }
}

impl crate::Draw for Dialog {
  fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    self.prompt.draw(w)?;
    match &self.input {
      DlgInput::Ack(r) | 
      DlgInput::Ask(r) => { 
        r.draw(w)?;
      }
      DlgInput::Edit(r) => {
        r.draw(w)?;
        r.cursor.draw(w)?;
      }
      DlgInput::Select(r) => {
        r.draw(w)?;
        r.cursor.draw(w)?;
      }
    }
    Ok(())
  }
}
