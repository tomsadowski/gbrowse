// src/layout.rs

use crate::{
  Rect, 
  Cursor,
  Frame,
  FrameParams,
  PointView,
  Page,
  PageParams,
};
use std::{
  collections::HashMap,
  rc::Rc,
};


pub trait GetHeight { 
  fn get_height(&self) -> u16; 
}

impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 { 
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}

impl GetHeight for Page {
  fn get_height(&self) -> u16 {
    self.matrix.get_height()
  }
}

impl GetHeight for Frame {
  fn get_height(&self) -> u16 {
    self.border_rect.height()
  }
}

pub struct PageViewParams {
  pub max_height:   Option<u16>,
  pub write_cursor: bool,
  pub frame_params: FrameParams,
  pub page_params:  Rc<PageParams>,
}
impl From<Rc<PageParams>> for PageViewParams {
  fn from(page_params: Rc<PageParams>) -> Self {
    Self {
      write_cursor: false,
      max_height:   None,
      frame_params: FrameParams::init(),
      page_params,
    }
  }
}
impl PageViewParams {
  pub fn with_write_cursor(mut self, write_cursor: bool) 
    -> Self { self.set_write_cursor(write_cursor); self }
  pub fn set_write_cursor(&mut self, write_cursor: bool) {
    self.write_cursor = write_cursor;
  }

  pub fn with_max_height(mut self, max_height: Option<u16>) 
    -> Self { self.set_max_height(max_height); self }
  pub fn set_max_height(&mut self, max_height: Option<u16>) {
    self.max_height = max_height;
  }

  pub fn with_frame_params(mut self, frame_params: FrameParams) 
    -> Self { self.set_frame_params(frame_params); self }
  pub fn set_frame_params(&mut self, frame_params: FrameParams) {
    self.frame_params = frame_params;
  }

  fn build_max_height(&self, rect: &Rect) -> u16 {
    self.max_height.unwrap_or(u16::MIN).min(rect.height())
  }

  fn build_max_rect(&self, rect: &Rect) -> Rect {
    let max_height = self.build_max_height(rect);
    rect.set_height(max_height)
  }

  fn build_max_frame(&self, rect: &Rect) -> Frame {
    self.frame_params.build_from_outer(
      &self.build_max_rect(rect)
    )
  }

  pub fn resize(
    &self, 
    page:       &Page, 
    point_view: &mut PointView, 
    rect:       &Rect
  ) {
    let mut frame = self.build_max_frame(rect);
    if page.get_height() < frame.inner_rect.height() {
      frame = self.frame_params.build_from_inner(
        &frame.inner_rect.set_height(page.get_height())
      );
    }
    point_view.resize(&page.point, &frame.inner_rect);
  }

  pub fn rebuild(
    &self, 
    page:       &mut Page, 
    point_view: &mut PointView, 
    rect:       &Rect
  ) {
    let mut frame = self.build_max_frame(rect);
    page.rebuild(&self.page_params, frame.inner_rect.width());
    if page.get_height() < frame.inner_rect.height() {
      frame = self.frame_params.build_from_inner(
        &frame.inner_rect.set_height(page.get_height())
      );
    }
    point_view.resize(&page.point, &frame.inner_rect);
  }

  pub fn build(self, rect: &Rect) -> PageView {
    let mut frame = self.build_max_frame(rect);
    let page = self.page_params.build(
      frame.inner_rect.width()
    );
    if page.get_height() < frame.inner_rect.height() {
      frame = self.frame_params.build_from_inner(
        &frame.inner_rect.set_height(page.get_height())
      );
    }
    let mut point_view = PointView::from(&frame.inner_rect);
    point_view.update(&page.point);
    PageView {
      point_view,
      frame,
      page,
      view_params: self,
    }
  }
}

pub struct PageView {
  pub view_params:  PageViewParams,
  pub point_view:   PointView,
  pub frame:        Frame,
  pub page:         Page,
}
impl PageView {
  pub fn rebuild(&mut self, rect: &Rect) {
    self.view_params.rebuild(&mut self.page, &mut self.point_view, rect);
  }

  // dont rewrap, only point_view changes
  pub fn resize(&mut self, rect: &Rect) {
    self.view_params.resize(&self.page, &mut self.point_view, rect);
  }
}

pub struct PageViewList {
  pub cursor:     Rc<Cursor>,
  pub page_views: Vec<PageView>,
}
impl PageViewList {
  pub fn rebuild(&mut self, rect: &Rect) {
    for view in self.page_views.iter_mut() {
      view.rebuild(rect);
    }
  }

  // dont rewrap, only point_view changes
  pub fn resize(&mut self, rect: &Rect) {
    for view in self.page_views.iter_mut() {
      view.resize(rect);
    }
  }
}

pub enum View {
  Page(PageView),
  List(PageViewList),
}
impl View {
  pub fn rebuild(&mut self, rect: &Rect) {
    match self {
      Self::Page(view) => view.rebuild(rect),
      Self::List(list) => list.rebuild(rect),
    }
  }

  // dont rewrap, only point_view changes
  pub fn resize(&mut self, rect: &Rect) {
    match self {
      Self::Page(view) => view.resize(rect),
      Self::List(list) => list.resize(rect),
    }
  }
}

pub struct Layout {
  pub max_rect:     Rect,
  pub frame_params: FrameParams,
  pub frame:        Frame,
  pub view_map:     HashMap<u16, View>,
}

impl From<Rect> for Layout {
  fn from(rect: Rect) -> Self {
    let frame = FrameParams::init().build_from_outer(&rect);
    Self {
      frame,
      max_rect:     rect,
      frame_params: FrameParams::init(),
      view_map:     HashMap::default(),
    }
  }
}

impl Layout {
  fn push_new_frame(&mut self) {
    let mut keys: Vec<u16> = self.view_map
      .keys().map(|k| k.clone()).collect();
    keys.sort();
    for k in keys.iter() {
      if let Some(value) = self.view_map.get_mut(k) {
        value.rebuild(&self.frame.inner_rect);
      }
    }
  }

  pub fn set_max_rect(&mut self, rect: Rect) {
    self.max_rect = rect;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.push_new_frame();
  }

  pub fn with_frame_params(mut self, frame_params: FrameParams) 
    -> Self { self.set_frame_params(frame_params); self }
  pub fn set_frame_params(&mut self, frame_params: FrameParams) {
    self.frame_params = frame_params;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.push_new_frame();
  }

  pub fn insert_page(&mut self, handle: u16, view_params: PageViewParams) {
    self.view_map.insert(
      handle, View::Page(view_params.build(&self.frame.inner_rect))
    );
  }

  pub fn remove(&mut self, handle: u16) {
  }
}
