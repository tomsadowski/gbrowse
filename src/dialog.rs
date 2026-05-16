// src/dialog.rs

use crate::{
  rect::Rect,
  style::Style,
  text::StyledText,
  widget::{TextBox, EditBox},
};

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
