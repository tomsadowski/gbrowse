// src/dlg.rs

use crate::{
  user,
  SystemParams, 
  TextParams,
  PageParams,
  FrameParams,
  Page,
  Rect,
  Frame,
  View,
  constants::*,
};


#[derive(Debug)]
pub enum DlgType {
  Ack, Ask, Edit, Select,
}


pub struct DialogParams<'a> {
  params: &'a SystemParams, 
  frame: FrameParams,
  dlg_type: DlgType,
  header: PageParams,
  body: PageParams,
}


impl<'a> From<&'a SystemParams> for DialogParams<'a> {
  fn from(user: &'a SystemParams) -> Self {
    Self {
      frame: FrameParams::default(),
      header: PageParams::default(),
      body: PageParams::default(),
      dlg_type: DlgType::Ack,
      params: user
    }
  }
}


impl<'a> DialogParams<'a> { 
  pub fn prompt(mut self, prompt: &str) -> Self {
    self.header = PageParams::init()
      .text(vec![prompt.to_string()])
      .style(&self.params.style.info)
      .max(2);
    self
  }


  pub fn ack(mut self) -> Self {
    self.body = PageParams::init()
      .text(vec![format!("Press any key to acknowledge")])
      .style(&self.params.style.info);
    self.dlg_type = DlgType::Ack;
    self
  }


  pub fn ask(mut self) -> Self {
    let guide = format!(
      "{} yes {} no", self.params.keys.yes, self.params.keys.no
    );
    self.body = PageParams::init()
      .text(vec![guide])
      .style(&self.params.style.info);
    self.dlg_type = DlgType::Ask;
    self
  }


  pub fn edit(mut self, text: &str) -> Self {
    self.body = PageParams::init()
      .text(vec![text])
      .style(&self.params.style.info)
      .edit(true)
      .draw_point(true);
    self.dlg_type = DlgType::Edit;
    self
  }


  pub fn select(mut self, options: Vec<String>) -> Self {
    self.body = PageParams::init()
      .text(options)
      .style(&self.params.style.info)
      .draw_point(true);
    self.dlg_type = DlgType::Select;
    self
  }
}

impl BuildView<Dialog> for DialogParams {
  fn build(self, rect: &Rect) -> Dialog {

  }
}


pub struct Dialog {
  pub frame: Frame,
  pub dlg_type: DlgType,
  pub header: Page<String>,
  pub body: Page<String>,
}


impl View for Dialog {
  fn get_height(&self) -> u16 {
    0
  }


  fn resize(&mut self, rect: &Rect) {
  }



  fn rebuild(&mut self, rect: &Rect) {
  }


  fn draw(&self, w: &mut std::io::Stdout) -> std::io::Result<()> {
    Ok(())
  }
}
