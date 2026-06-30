// src/app.rs

use crate::{
  gemini, 
  util,
  Dim,
  user,
  User, 
  CursorVec,
  Tab,
  TextStyle,
  UserTable,
  user_from_str,
  Request,
  Layout,
  Action,
  Rect, 
  PageViewParams,
  PageParams,
  GemTag, 
  Status, 
  StatusText,
  Frame,
  constants::*,
};
struct Dlg {
}
impl Dlg { 
  fn get_dlg_params(&self, layout: &mut Layout, prompt: &[String], text: &[String])
    -> (PageViewParams, PageViewParams) 
  {
    let dlg_1 = PageViewParams::from(
      PageParams::init()
        .with_text(prompt)
        .with_style(user.style.info)
    );
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(text)
        .with_style(user.style.info)
    ).with_frame_params(user.get_dialog_frame_params());
    (dlg_1, dlg_2)
  }

  fn focus_ack_dialog(&mut self, layout: &mut Layout, prompt: String) {
    guide = format!("Press any key to acknowledge");
    let (dlg_1, dlg_2) = get_dlg_params(
      &vec![prompt], &vec![guide.clone()]
    );
    layout.insert(DLG_1, dlg_1);
    layout.insert(DLG_2, dlg_2);
  }

  fn focus_ask_dialog(&mut self, layout: &mut Layout, prompt: &str) {
    guide = format!(
      "{} yes {} no", user.keys.yes, user.keys.no
    );
    let (dlg_1, dlg_2) = get_dlg_params(
      &vec![prompt.into()], &vec![guide.clone()]
    );
    layout.insert(DLG_1, dlg_1);
    layout.insert(DLG_2, dlg_2);
  }

  fn focus_edit_dialog(&mut self, layout: &mut Layout, prompt: &str, text: &str) {
    guide = format!("Press {} to cancel", user.keys.cancel);
    let (dlg_1, dlg_2) = get_dlg_params(
      &vec![prompt.into()], &vec![text.into()]
    );
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![text])
        .with_style(user.style.info)
        .edit(true)
    )
    .with_frame_params(user.get_dialog_frame_params())
    .with_draw_point(true);
    layout.insert(DLG_1, dlg_1);
    layout.insert(DLG_2, dlg_2);
  }

  fn focus_select_dialog(
    &mut self, 
    layout: &mut Layout
    prompt: &str, 
    options: Vec<String>
  ) {
    let (dlg_1, mut dlg_2) = get_dlg_params(
      &vec![prompt.into()], &options
    );
    dlg_2.set_draw_point(true);
    layout.insert(DLG_1, dlg_1);
    layout.insert(DLG_2, dlg_2);
    guide = format!("Press {} to select", user.keys.select);
  }
}
