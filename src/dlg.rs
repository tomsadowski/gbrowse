// src/dlg.rs

use crate::{
  SystemParams, 
  FrameParams,
  Frame,
  PageParams,
  Page,
  Rect,
  GetDisplayHeight,
  resize_views,
  build_opt_views,
  Draw,
  Resize,
  BuildView,
  GetMaxHeight,
};


#[derive(Debug)]
pub enum DlgType {
  Ack, 
  Ask, 
  Edit, 
  Select,
  //Flash,
}


pub struct DialogParams<'a> {
  params: &'a SystemParams, 
  frame: FrameParams,
  dlg_type: DlgType,
  header: Option<PageParams<String>>,
  body: Option<PageParams<String>>,
}


impl<'a> From<&'a SystemParams> for DialogParams<'a> {
  fn from(params: &'a SystemParams) -> Self {
    Self {
      frame: params.style.get_dialog_frame_params(),
      header: None,
      body: None,
      dlg_type: DlgType::Ack,
      params
    }
  }
}


impl<'a> DialogParams<'a> { 
  pub fn prompt(mut self, prompt: &str) -> Self {
    self.header = Some(
      PageParams::init()
        .text(vec![prompt.to_string()])
        .style(&self.params.style.info)
        .max(2)
    );
    self
  }


  pub fn ack(mut self) -> Self {
    self.body = Some(
      PageParams::init()
        .text(vec![format!("Press any key to acknowledge")])
        .style(&self.params.style.info)
    );
    self.dlg_type = DlgType::Ack;
    self
  }


  pub fn ask(mut self) -> Self {
    let guide = format!(
      "{} yes {} no", self.params.keys.yes, self.params.keys.no
    );
    self.body = Some(
      PageParams::init()
        .text(vec![guide])
        .style(&self.params.style.info)
    );
    self.dlg_type = DlgType::Ask;
    self
  }


  pub fn edit(mut self, text: &str) -> Self {
    self.body = Some(
      PageParams::init()
        .text(vec![text.to_string()])
        .style(&self.params.style.info)
        .edit(true)
    );
    self.dlg_type = DlgType::Edit;
    self
  }


  pub fn select(mut self, options: Vec<String>) -> Self {
    self.body = Some(
      PageParams::init()
        .text(options)
        .style(&self.params.style.info)
    );
    self.dlg_type = DlgType::Select;
    self
  }
}



impl<'a> BuildView<Dialog> for DialogParams<'a> {
  fn build(self, rect: &Rect) -> Dialog {
    let frame = self.frame.build_from_outer(rect);
    let mut views = build_opt_views(
      &frame.inner_rect, vec![self.header, self.body]
    );

    let mut inner = frame.inner_rect.clone();
    inner.h = views.iter().map(|v| v.get_display_height()).sum();
    let frame = self.frame.build_from_inner(&inner);

    Dialog {
      frame, 
      dlg_type: self.dlg_type, 
      body: views.pop().unwrap(),
      header: views.pop().unwrap(),
    }
  }
}


pub struct Dialog {
  pub frame: Frame,
  pub dlg_type: DlgType,
  pub header: Option<Page<String>>,
  pub body: Option<Page<String>>,
}


impl GetMaxHeight for Dialog {
  fn get_max_height(&self) -> u16 {
    self.header.get_max_height() 
      + self.body.get_max_height()
      + (self.frame.screen.h - self.frame.inner_rect.h)
  }
}


impl GetDisplayHeight for Dialog {
  fn get_display_height(&self) -> u16 {
    self.frame.get_display_height()
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
    resize_views(
      &frame.inner_rect, 
      &mut vec![&mut self.header, &mut self.body]
    );

    let mut inner_rect = frame.inner_rect.clone();
    inner_rect.h = 
      self.header.get_display_height() 
      + self.body.get_display_height();
    self.frame = self.frame.params.build_from_inner(&inner_rect);
  }
}
