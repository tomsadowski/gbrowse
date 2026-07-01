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


pub struct Dlg<'a>(pub &'a User, pub &'a mut Layout);

impl<'a> Dlg<'a> { 
  pub fn prompt(&mut self, prompt: &str) -> &mut Self {
    self.1.insert(
      DLG_1, 
      PageParams::init()
        .with_text(&vec![prompt.to_string()])
        .with_style(self.0.style.info).into()
    );
    self
  }

  pub fn ack(&mut self) {
    let guide = format!("Press any key to acknowledge");
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![guide])
        .with_style(self.0.style.info)
    ).with_frame_params(self.0.get_dialog_frame_params());
    self.1.insert(DLG_2, dlg_2);
  }

  pub fn ask(&mut self) {
    let guide = format!(
      "{} yes {} no", self.0.keys.yes, self.0.keys.no
    );
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![guide])
        .with_style(self.0.style.info)
    ).with_frame_params(self.0.get_dialog_frame_params());
    self.1.insert(DLG_2, dlg_2);
  }

  pub fn edit(&mut self, text: &str) {
    self.1.insert(
      DLG_2, 
      PageViewParams::from(
        PageParams::init()
          .with_text(&vec![text])
          .with_style(self.0.style.info)
          .edit(true)
      )
      .with_frame_params(self.0.get_dialog_frame_params())
      .with_draw_point(true)
    );
  }

  pub fn select(&mut self, options: Vec<String>) {
    self.1.insert(
      DLG_2, 
      PageViewParams::from(
        PageParams::init()
          .with_text(&options)
          .with_style(self.0.style.info)
        )
        .with_frame_params(self.0.get_dialog_frame_params())
        .with_draw_point(true)
    );
  }
}
