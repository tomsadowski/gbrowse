// src/view.rs

use crate::{
  PageParams,
  FrameParams,
  Rect,
  Frame,
  fill,
  resize_views,
  Tab,
  TabText,
  GetDisplayHeight,
  Dialog,
  DialogParams,
  CursorVec,
  ViewType,
  ViewTypeMut,
  BuildView,
};


pub struct AppView {
  pub draw_frame: bool,
  pub rect: Rect,
  pub frame: Frame,
  pub flash: Option<Dialog>,
  pub dialog: Option<Dialog>,
  pub tabs: CursorVec<Tab>,
}


impl crate::Draw for AppView {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    if self.draw_frame {
      self.frame.draw_east(w)?;
      self.frame.draw_west(w)?;
    }
    for view in self.get_view_list() {
      view.draw(w)?;
    }
    fill(&self.rect, &self.frame, w)?;
    Ok(())
  }
}


impl crate::Resize for AppView {
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
    self.tabs.insert_unique_with(
      |tab| &tab.url == url,
      Tab {
        url: url.clone(), 
        page: params.build(&self.frame.inner_rect)
      },
    );
    self.push_frame();
  }


  pub fn get_view_list<'a>(&'a self) -> Vec<Option<ViewType<'a, TabText>>> {
    vec![
      self.flash.as_ref().map(ViewType::Dialog),
      self.dialog.as_ref().map(ViewType::Dialog),
      self.tabs.get().map(|f| &f.page).map(ViewType::Page),
    ]
  }


  pub fn get_view_list_mut<'a>(&'a mut self) 
    -> Vec<Option<ViewTypeMut<'a, TabText>>> 
  {
    vec![
      self.flash.as_mut().map(ViewTypeMut::Dialog),
      self.dialog.as_mut().map(ViewTypeMut::Dialog),
      self.tabs.get_mut().map(|f| &mut f.page).map(ViewTypeMut::Page),
    ]
  }
}
