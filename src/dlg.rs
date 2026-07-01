// src/app.rs

use crate::{
  user,
  User, 
  TextStyle,
  Layout,
  PageViewParams,
  PageParams,
  Frame,
  constants::*,
};


pub struct Dlg<'a> {
  user:   &'a User, 
  header: PageViewParams,
  body:   PageViewParams,
}

impl<'a> From<&'a User> for Dlg<'a> {
  fn from(user: &'a User) -> Self {
    Self {
      header: PageViewParams::default(),
      body:   PageViewParams::default(),
      user
    }
  }
}

impl<'a> Dlg<'a> { 
  pub fn add(self, layout: &mut Layout) {
    layout.insert(DLG_1, self.header);
    layout.insert(DLG_2, self.body);
  }

  pub fn prompt(mut self, prompt: &str) -> Self {
    self.header = PageParams::init()
      .with_text(&vec![prompt.to_string()])
      .with_style(self.user.style.info)
      .into();
    self
  }

  pub fn ack(mut self) -> Self {
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![format!("Press any key to acknowledge")])
        .with_style(self.user.style.info)
    ).with_frame_params(self.user.get_dialog_frame_params());
    self
  }

  pub fn ask(mut self) -> Self {
    let guide = format!(
      "{} yes {} no", self.user.keys.yes, self.user.keys.no
    );
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![guide])
        .with_style(self.user.style.info)
    ).with_frame_params(self.user.get_dialog_frame_params());
    self
  }

  pub fn edit(mut self, text: &str) -> Self {
    self.body = 
      PageViewParams::from(
        PageParams::init()
          .with_text(&vec![text])
          .with_style(self.user.style.info)
          .edit(true)
      )
      .with_frame_params(self.user.get_dialog_frame_params())
      .with_draw_point(true);
    self
  }

  pub fn select(mut self, options: Vec<String>) -> Self {
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&options)
        .with_style(self.user.style.info)
      )
      .with_frame_params(self.user.get_dialog_frame_params())
      .with_draw_point(true);
    self
  }
}
