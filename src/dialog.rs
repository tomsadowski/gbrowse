// src/dialog.rs

use crate::{
  Page, 
  Style, 
  Rect,
  Layout,
};


pub enum DlgInput {
  Text  (Page),
  Select(Page),
  Edit  (Page),
}

pub struct Dialog {
  pub layout: Layout,
  pub prompt: Page,
  pub input:  DlgInput,
} 

impl Dialog {
  pub fn ack<S>(rect: &Rect, style: S, prompt: &str, input: &str) -> Self
  where S: Into<Style> + Copy
  {
    let prompt_box = Page::from(rect)
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = Page::from(rect)
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

  pub fn ask<S>(rect: &Rect, style: S, prompt: &str, input: &str) -> Self 
  where S: Into<Style> + Copy
  {
    let prompt_box = Page::from(
        rect
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = Page::from(
        rect
      )
      .text(
        vec![input.into()], 
        vec![]
      )
      .style(style);
    Dialog {
      prompt: prompt_box,
      input:  DlgInput::Text(response_box),
    }
  }

  pub fn select<S>(rect: &Rect, style: S, prompt: &str, input: Vec<String>) 
    -> Self 
  where S: Into<Style> + Copy
  {
    let prompt_box = Page::from(
        rect
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = Page::from(
        rect
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

  pub fn edit<S>(rect: &Rect, style: S, prompt: &str, text: &str) -> Self 
  where S: Into<Style> + Copy
  {
    let prompt_box = Page::from(
        rect
      )
      .text(
        vec![prompt.into()], 
        vec![]
      )
      .style(style);
    let response_box = Page::from(
        rect
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

  pub fn resize(&mut self, rect: &Rect) {
    self.prompt.resize(&rect);
    match &mut self.input {
      DlgInput::Ack(r) | 
      DlgInput::Text(r) |
      DlgInput::Edit(r) => {
        r.resize(self.prompt.used_rect().bottom_row());
      }
      DlgInput::Select(r) => {
        r.resize(rect.get_rect().crop_north(self.prompt.used_rect().h));
      }
    }
  }

  pub fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    self.prompt.draw(w)?;
    match &self.input {
      DlgInput::Ack(r) | 
      DlgInput::Text(r) => { 
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
