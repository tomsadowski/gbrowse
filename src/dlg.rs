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

#[derive(Debug)]
pub enum DlgType {
  Ack, Ask, Edit, Select,
}

pub fn remove(layout: &mut Layout) {
  layout.remove_list(DLG_1);
  layout.remove_list(DLG_2);
}

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
  pub fn prompt(mut self, prompt: &str) -> Self {
    self.header = PageParams::init()
      .with_text(&vec![prompt.to_string()])
      .with_style(self.user.style.info)
      .into();
    self
  }

  pub fn ack(mut self, layout: &mut Layout) -> DlgType {
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![format!("Press any key to acknowledge")])
        .with_style(self.user.style.info)
    ).with_frame_params(self.user.style.get_dialog_frame_params());
    layout.insert(DLG_1, self.header);
    layout.insert(DLG_2, self.body);
    DlgType::Ack
  }

  pub fn ask(mut self, layout: &mut Layout) -> DlgType {
    let guide = format!(
      "{} yes {} no", self.user.keys.yes, self.user.keys.no
    );
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![guide])
        .with_style(self.user.style.info)
    ).with_frame_params(self.user.style.get_dialog_frame_params());
    layout.insert(DLG_1, self.header);
    layout.insert(DLG_2, self.body);
    DlgType::Ask
  }

  pub fn edit(mut self, text: &str, layout: &mut Layout) -> DlgType {
    self.body = 
      PageViewParams::from(
        PageParams::init()
          .with_text(&vec![text])
          .with_style(self.user.style.info)
          .edit(true)
      )
      .with_frame_params(self.user.style.get_dialog_frame_params())
      .with_draw_point(true);
    layout.insert(DLG_1, self.header);
    layout.insert(DLG_2, self.body);
    DlgType::Edit
  }

  pub fn select(mut self, options: Vec<String>, layout: &mut Layout) 
  -> DlgType {
    self.body = PageViewParams::from(
      PageParams::init()
        .with_text(&options)
        .with_style(self.user.style.info)
      )
      .with_frame_params(self.user.style.get_dialog_frame_params())
      .with_draw_point(true);
    layout.insert(DLG_1, self.header);
    layout.insert(DLG_2, self.body);
    DlgType::Select
  }
}
