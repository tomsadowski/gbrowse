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
  pub draw_frame: bool,
  pub rect: Rect,
  pub frame: Frame,
  pub flash: Option<Dialog>,
  pub dialog: Option<Dialog>,
  pub tabs: CursorVec<Tab>,
}


impl Draw for AppView {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    if self.draw_frame {
      self.frame.draw(w)?;
    }
    for view in self.get_view_list() {
      view.draw(w)?;
    }
    fill(&self.rect, &self.frame, w)?;
    Ok(())
  }
}


impl Resize for AppView {
  fn resize(&mut self, rect: &Rect) {
    self.rect = rect.clone();
    self.frame = self.frame.params.build_from_outer(rect);
    self.push_frame();
  }
}


impl AppView {
  pub fn new(rect: &Rect, params: &FrameParams) -> Self {
    let mut appview = Self {
      draw_frame: true,
      frame: params.build_from_outer(rect),
      flash: None,
      dialog: None,
      tabs: CursorVec::default(),
      rect: rect.clone(),
    };
    appview.push_frame();
    appview
  }


  pub fn reset_frame(&mut self) {
    self.frame = self.frame.params.build_from_outer(&self.rect);
    self.push_frame();
  }


  pub fn reset_draw_state(&mut self) {
    self.draw_frame = false;
  }


  pub fn push_frame(&mut self) {
    self.draw_frame = true;
    let mut inner_rect = self.frame.inner_rect;
    resize_views(
      &inner_rect, 
      &mut self.get_view_list_mut().iter_mut().collect()
    );
    inner_rect.h = self
      .get_view_list()
      .iter()
      .map(|v| v.get_display_height())
      .sum();
    self.frame = self.frame.params.build_from_inner(&inner_rect);
  }

  
  pub fn flash(&mut self, params: DialogParams) {
    self.frame = self.frame.params.build_from_outer(&self.rect);
    self.flash = Some(params.build(&self.frame.inner_rect));
    self.push_frame();
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


  pub fn get_view_list<'a>(&'a self) -> Vec<Option<ViewType<'a>>> {
    vec![
      self.flash.as_ref().map(ViewType::Dialog),
      self.dialog.as_ref().map(ViewType::Dialog),
      self.tabs
        .get()
        .map(|f| &f.page)
        .map(ViewType::Tab),
    ]
  }


  pub fn get_view_list_mut<'a>(&'a mut self) 
    -> Vec<Option<ViewTypeMut<'a>>> 
  {
    vec![
      self.flash.as_mut().map(ViewTypeMut::Dialog),
      self.dialog.as_mut().map(ViewTypeMut::Dialog),
      self.tabs
        .get_mut()
        .map(|f| &mut f.page)
        .map(ViewTypeMut::Tab),
    ]
  }
}


pub enum ViewType<'a> {
  Dialog(&'a Dialog),
  Tab(&'a Page<TabText>),
}
impl<'a> GetMaxHeight for ViewType<'a> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Tab(page) => page.get_max_height(),
    }
  }
}
impl<'a> GetDisplayHeight for ViewType<'a> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Tab(page) => page.get_display_height(),
    }
  }
}
impl<'a> Draw for ViewType<'a> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Tab(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}


pub enum ViewTypeMut<'a> {
  Dialog(&'a mut Dialog),
  Tab(&'a mut Page<TabText>),
}
impl<'a> GetDisplayHeight for ViewTypeMut<'a> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Tab(page) => page.get_display_height(),
    }
  }
}
impl<'a> GetMaxHeight for ViewTypeMut<'a> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Tab(page) => page.get_max_height(),
    }
  }
}
impl<'a> Draw for ViewTypeMut<'a> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Tab(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}
impl<'a> Resize for ViewTypeMut<'a> {
  fn resize(&mut self, rect: &Rect) {
    match self {
      Self::Dialog(dialog) => dialog.resize(rect),
      Self::Tab(page) => page.resize(rect),
    }
  }
}
