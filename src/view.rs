// src/view.rs

use crate::{
  PageParams,
  FrameParams,
  Rect,
  Draw,
  Frame,
  fill,
  resize_views,
  Tab,
  TabText,
//  get_display_heights,
  GetDisplayHeight,
  Dialog,
  DialogParams,
  CursorVec,
  Resize,
  GetMaxHeight,
  BuildView,
  Page,
};



pub struct AppView {
  pub rect: Rect,
  pub frame: Frame,
  pub flash: Option<Page<String>>,
  pub dialog: Option<Dialog>,
  pub tabs: CursorVec<Tab>,
}


impl Draw for AppView {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    self.frame.draw(w)?;
    for view in self.get_view_list() {
      view.draw(w)?;
    }
    fill(&self.rect, &self.frame, w)?;
    Ok(())
  }
}


impl Resize for AppView {
  fn resize(&mut self, rect: &Rect) {
    self.frame = self.frame.params.build_from_outer(rect);
    self.push_frame();
  }
}


impl AppView {
  pub fn new(rect: &Rect, params: &FrameParams) -> Self {
    let mut appview = Self {
      frame: params.build_from_outer(rect),
      flash: None,
      dialog: None,
      tabs: CursorVec::default(),
      rect: rect.clone(),
    };
    appview.push_frame();
    appview
  }


  pub fn push_frame(&mut self) {
    let mut inner_rect = self.frame.inner_rect;
    resize_views(
      &inner_rect, 
      &mut self.get_view_list_mut().iter_mut().collect()
    );
   // let mut inner = frame.inner_rect.clone();
   // inner.h = get_heights(&inner, &views).iter().sum();
    inner_rect.h = self
      .get_view_list()
      .iter()
      .map(|v| v.get_display_height())
      .sum();
    self.frame = self.frame.params.build_from_inner(&inner_rect);
  }


  pub fn dialog(&mut self, params: DialogParams) {
    self.frame = self.frame.params.build_from_outer(&self.rect);
    self.dialog = Some(params.build(&self.frame.inner_rect));
    self.push_frame();
  }


  pub fn tab(&mut self, url: &url::Url, params: PageParams<TabText>) {
    self.frame = self.frame.params.build_from_outer(&self.rect);
    self.tabs.insert(Tab {
      url: url.clone(), 
      page: params.build(&self.frame.inner_rect) 
    });
    self.push_frame();
  }


  pub fn get_view_list<'a>(&'a self) -> Vec<ViewType<'a>> {
    let mut vec: Vec<ViewType> = vec![];
    if let Some(flash) = &self.flash {
      vec.push(ViewType::Flash(flash));
    }
    if let Some(dlg) = &self.dialog {
      vec.push(ViewType::Dialog(dlg));
    }
    if let Some(tab) = self.tabs.get_current() {
      vec.push(ViewType::Tab(&tab.page));
    }
    vec
  }


  pub fn get_view_list_mut<'a>(&'a mut self) -> Vec<ViewTypeMut<'a>> {
    let mut vec: Vec<ViewTypeMut> = vec![];
    if let Some(flash) = &mut self.flash {
      vec.push(ViewTypeMut::Flash(flash));
    }
    if let Some(dlg) = &mut self.dialog {
      vec.push(ViewTypeMut::Dialog(dlg));
    }
    if let Some(tab) = self.tabs.get_current_mut() {
      vec.push(ViewTypeMut::Tab(&mut tab.page));
    }
    vec
  }
}


pub enum ViewType<'a> {
  Flash(&'a Page<String>),
  Dialog(&'a Dialog),
  Tab(&'a Page<TabText>),
}


impl<'a> GetMaxHeight for ViewType<'a> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Flash(flash) => flash.get_max_height(),
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Tab(page) => page.get_max_height(),
    }
  }
}


impl<'a> GetDisplayHeight for ViewType<'a> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Flash(flash) => flash.get_display_height(),
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Tab(page) => page.get_display_height(),
    }
  }
}


impl<'a> Draw for ViewType<'a> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Flash(flash) => {flash.draw(w)?;}
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Tab(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}


pub enum ViewTypeMut<'a> {
  Flash(&'a mut Page<String>),
  Dialog(&'a mut Dialog),
  Tab(&'a mut Page<TabText>),
}


impl<'a> GetDisplayHeight for ViewTypeMut<'a> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Flash(flash) => flash.get_display_height(),
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Tab(page) => page.get_display_height(),
    }
  }
}


impl<'a> GetMaxHeight for ViewTypeMut<'a> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Flash(flash) => flash.get_max_height(),
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Tab(page) => page.get_max_height(),
    }
  }
}


impl<'a> Draw for ViewTypeMut<'a> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Flash(flash) => {flash.draw(w)?;}
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Tab(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}


impl<'a> Resize for ViewTypeMut<'a> {
  fn resize(&mut self, rect: &Rect) {
    match self {
      Self::Flash(flash) => flash.resize(rect),
      Self::Dialog(dialog) => dialog.resize(rect),
      Self::Tab(page) => page.resize(rect),
    }
  }
}
