// src/dlg.rs

use crate::{
  user,
  SystemParams, 
  TextParams,
  FrameParams,
  Frame,
  PageParams,
  Page,
  Rect,
  get_heights,
  resize_views,
  build_views,
  Draw,
  Resize,
  BuildView,
  GetHeight,
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
  header: PageParams<String>,
  body: PageParams<String>,
}


impl<'a> From<&'a SystemParams> for DialogParams<'a> {
  fn from(params: &'a SystemParams) -> Self {
    Self {
      frame: params.style.get_dialog_frame_params(),
      header: PageParams::default(),
      body: PageParams::default(),
      dlg_type: DlgType::Ack,
      params
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
      .text(vec![text.to_string()])
      .style(&self.params.style.info)
      .edit(true);
    self.dlg_type = DlgType::Edit;
    self
  }


  pub fn select(mut self, options: Vec<String>) -> Self {
    self.body = PageParams::init()
      .text(options)
      .style(&self.params.style.info);
    self.dlg_type = DlgType::Select;
    self
  }
}


impl<'a> BuildView<Dialog> for DialogParams<'a> {
  fn build(self, rect: &Rect) -> Dialog {
    let frame = self.frame.build_from_outer(rect);
    let mut views = build_views(
      &frame.inner_rect, vec![self.header, self.body]
    );

    let mut inner = frame.inner_rect.clone();
    inner.h = get_heights(&views);
    let frame = self.frame.build_from_inner(&inner);

    Dialog {
      frame, 
      dlg_type: self.dlg_type, 
      header: views.pop().unwrap(),
      body: views.pop().unwrap(),
    }
  }
}


pub struct Dialog {
  pub frame: Frame,
  pub dlg_type: DlgType,
  pub header: Page<String>,
  pub body: Page<String>,
}


impl GetHeight for Dialog {
  fn get_height(&self) -> u16 {
    self.frame.screen.h
  }
}


impl Draw for Dialog {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    self.frame.draw(w)?;
    self.header.draw(w)?;
    self.body.draw(w)?;
    Ok(())
  }
}


impl Resize for Dialog {
  fn resize(&mut self, rect: &Rect) {
    let frame = self.frame.params.build_from_outer(rect);
    resize_views(&frame.inner_rect, vec![&mut self.header, &mut self.body]);

    let mut inner = frame.inner_rect.clone();
    inner.h = self.header.get_height() + self.body.get_height();
    self.frame = self.frame.params.build_from_inner(&inner);
  }
}
