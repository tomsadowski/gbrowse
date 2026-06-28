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

impl GetHeight for PageView {
  fn get_height(&self) -> u16 {
    self.frame.border_rect.height()
  }
}

impl GetHeight for PageViewList {
  fn get_height(&self) -> u16 {
    self.views
      .get(*self.cursor)
      .map(|page_view| page_view.get_height())
      .unwrap_or(u16::MIN)
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

#[derive(Default)]
pub struct PageViewList {
  pub cursor: Cursor,
  pub views:  Vec<PageView>,
}

impl From<PageView> for PageViewList {
  fn from(view: PageView) -> Self {
    let mut view_list = Self::default();
    view_list.insert(view);
    view_list
  }
}

impl PageViewList {
  pub fn insert(&mut self, view: PageView) {
    self.cursor.insert(&mut self.views, view);
  }

  pub fn rebuild(&mut self, rect: &Rect) {
    for view in self.views.iter_mut() {
      view.rebuild(rect);
    }
  }

  // dont rewrap, only point_view changes
  pub fn resize(&mut self, rect: &Rect) {
    for view in self.views.iter_mut() {
      view.resize(rect);
    }
  }
}

pub struct Layout {
  pub max_rect:     Rect,
  pub frame_params: FrameParams,
  pub frame:        Frame,
  pub view_map:     HashMap<u16, PageViewList>,
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
  fn get_sorted_keys(&self) -> Vec<u16> {
    let mut keys: Vec<_> = self.view_map.keys().map(|k| k.clone()).collect();
    keys.sort();
    keys
  }

  fn for_each<F: FnMut(&mut PageViewList)>(&mut self, mut func: F) {
    for k in self.get_sorted_keys().iter() {
      if let Some(view_list) = self.view_map.get_mut(k) {
        func(view_list);
      }
    }
  }

  fn resize(&mut self) {
    let mut rect = self.frame.inner_rect;
    self.for_each(|view_list| {
      view_list.resize(&rect);
      rect = rect.shift_north((view_list.get_height() as i16) * -1);
    });
  }

  fn rebuild(&mut self) {
    let mut rect = self.frame.inner_rect;
    self.for_each(|view_list| {
      view_list.rebuild(&rect);
      rect = rect.shift_north((view_list.get_height() as i16) * -1);
    });
  }

  fn get_rect_for_key(&self, key: u16) -> Rect {
    let mut rect = self.frame.inner_rect;
    for k in self.get_sorted_keys().iter().take_while(|k| **k < key) {
      if let Some(view_list) = self.view_map.get(k) {
        rect = rect.shift_north((view_list.get_height() as i16) * -1);
      }
    }
    for k in self.get_sorted_keys().iter().rev().take_while(|k| **k > key) {
      if let Some(view_list) = self.view_map.get(k) {
        rect = rect.shift_south((view_list.get_height() as i16) * -1);
      }
    }
    rect
  }

  pub fn set_max_rect(&mut self, rect: Rect) {
    self.max_rect = rect;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.rebuild();
  }

  pub fn with_frame_params(mut self, frame_params: FrameParams) 
    -> Self { self.set_frame_params(frame_params); self }

  pub fn set_frame_params(&mut self, frame_params: FrameParams) {
    self.frame_params = frame_params;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.rebuild();
  }

  pub fn insert(&mut self, key: u16, view_params: PageViewParams) {
    let rect = self.get_rect_for_key(key);
    let view = view_params.build(&rect);
    if let Some(view_list) = self.view_map.get_mut(&key) {
      view_list.insert(view);
    } else {
      self.view_map.insert(key, view.into());
    }
  }

  pub fn remove(&mut self, handle: u16) {
  }
}
