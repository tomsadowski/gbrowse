// src/layout.rs

use crate::{
  Rect, 
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
  pub fn set_write_cursor(&mut self, write_cursor: bool) {
    self.write_cursor = write_cursor;
  }
  pub fn with_write_cursor(mut self, write_cursor: bool) -> Self {
    self.set_write_cursor(write_cursor);
    self
  }
  pub fn set_max_height(&mut self, max_height: Option<u16>) {
    self.max_height = max_height;
  }
  pub fn with_max_height(mut self, max_height: Option<u16>) -> Self {
    self.set_max_height(max_height);
    self
  }
  pub fn set_frame_params(&mut self, frame_params: FrameParams) {
    self.frame_params = frame_params;
  }
  pub fn with_frame_params(mut self, frame_params: FrameParams) -> Self {
    self.set_frame_params(frame_params);
    self
  }
  pub fn build(&self, rect: &Rect) -> PageView {
  }
}

pub struct PageView {
  pub view_params:  PageViewParams,
  pub point_view:   PointView,
  pub frame:        Frame,
  pub page:         Page,
}


pub enum View {
  Layout(Rc<Layout>),
  Page(PageView),
  List(Vec<PageView>),
}

pub struct Layout {
  pub max_rect:     Rect,
  pub frame_params: FrameParams,
  pub view_map:     HashMap<u16, View>,
}

impl From<&Rect> for Layout {
  fn from(rect: &Rect) -> Self {
    Self {
      max_rect:     rect.clone(),
      frame_params: FrameParams::init(),
      view_map:     HashMap::default(),
    }
  }
}

impl Layout {
  pub fn set_frame_params(&mut self, frame_params: FrameParams) {
    self.frame_params = frame_params;
  }
  pub fn with_frame_params(mut self, frame_params: FrameParams) -> Self {
    self.set_frame_params(frame_params);
    self
  }
  // add
  pub fn insert_page(&mut self, handle: u16, page_params: PageParams) {
//    self.view_map.insert(handle, View::Page(page_params.into()));
  }
  // remove
  pub fn remove(&mut self, handle: u16) {
  }
}
