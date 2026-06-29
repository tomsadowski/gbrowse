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
use std::collections::HashMap;


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
  pub draw_point: bool,
  pub frame_params: FrameParams,
  pub page_params:  PageParams,
}
impl From<PageParams> for PageViewParams {
  fn from(page_params: PageParams) -> Self {
    Self {
      draw_point: false,
      max_height:   None,
      frame_params: FrameParams::init(),
      page_params,
    }
  }
}
impl PageViewParams {
  pub fn draw<W: std::io::Write>(
    &self, 
    page:   &Page, 
    view:   &PointView, 
    writer: &mut W
  ) -> std::io::Result<()> {
    self.page_params.draw(page, view, writer)?;
    if self.draw_point {
      view.draw(writer)?;
    }
    Ok(())
  }

  pub fn with_draw_point(mut self, b: bool) 
    -> Self { self.set_draw_point(b); self }

  pub fn set_draw_point(&mut self, b: bool) {
    self.draw_point = b;
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
  pub fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    self.view_params.draw(&self.page, &self.point_view, writer)?;
    Ok(())
  }

  pub fn get_param_string(&self) -> &str {
    self.view_params.page_params.get_string(&self.page)
  }

  pub fn get_page_string(&self) -> Option<String> {
    self.page.get_string()
  }

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
  pub fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    if let Some(view) = self.get_page_view() {
      view.draw(writer)?;
    }
    Ok(())
  }

  pub fn get_page_view(&self) -> Option<&PageView> {
    self.views.get(*self.cursor)
  }

  pub fn get_page_view_mut(&mut self) -> Option<&mut PageView> {
    self.views.get_mut(*self.cursor)
  }

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
  pub map:          HashMap<u16, PageViewList>,
}

impl From<Rect> for Layout {
  fn from(rect: Rect) -> Self {
    let frame = FrameParams::init().build_from_outer(&rect);
    Self {
      frame,
      max_rect:     rect,
      frame_params: FrameParams::init(),
      map:          HashMap::default(),
    }
  }
}

impl Layout {
  pub fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
    self.for_each_sorted(|v| v.draw(writer));
    Ok(())
  }

  pub fn get_page_view_mut(&mut self, key: u16) -> Option<&mut PageView> {
    self.map
      .get_mut(&key)
      .and_then(|v| v.get_page_view_mut())
  }

  fn get_sorted_keys(&self) -> Vec<u16> {
    let mut keys: Vec<_> = self.map.keys().map(|k| k.clone()).collect();
    keys.sort();
    keys
  }

  fn for_each_sorted<F, T>(&self, mut func: F)
  where F: FnMut(&PageViewList) -> T
  {
    for k in self.get_sorted_keys().iter() {
      if let Some(v) = self.map.get(k) { func(v); }
    }
  }

  fn for_each_sorted_mut<F>(&mut self, mut func: F) 
  where F: FnMut(&mut PageViewList)
  {
    for k in self.get_sorted_keys().iter() {
      if let Some(v) = self.map.get_mut(k) { func(v); }
    }
  }

  fn push_resize(&mut self) {
    let mut rect = self.frame.inner_rect;
    self.for_each_sorted_mut(|view_list| {
      view_list.resize(&rect);
      rect = rect.shift_north((view_list.get_height() as i16) * -1);
    });
  }

  fn push_rebuild(&mut self) {
    let mut rect = self.frame.inner_rect;
    self.for_each_sorted_mut(|view_list| {
      view_list.rebuild(&rect);
      rect = rect.shift_north((view_list.get_height() as i16) * -1);
    });
  }

  fn get_rect_for_key(&self, key: u16) -> Rect {
    let mut rect = self.frame.inner_rect;
    for k in self.get_sorted_keys().iter().take_while(|k| **k < key) {
      if let Some(view_list) = self.map.get(k) {
        rect = rect.shift_north((view_list.get_height() as i16) * -1);
      }
    }
    for k in self.get_sorted_keys().iter().rev().take_while(|k| **k > key) {
      if let Some(view_list) = self.map.get(k) {
        rect = rect.shift_south((view_list.get_height() as i16) * -1);
      }
    }
    rect
  }

  pub fn set_max_rect(&mut self, rect: Rect) {
    self.max_rect = rect;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.push_rebuild();
  }

  pub fn with_frame_params(mut self, params: FrameParams) 
    -> Self { self.set_frame_params(params); self }

  pub fn set_frame_params(&mut self, params: FrameParams) {
    self.frame_params = params;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.push_rebuild();
  }

  pub fn resize(&mut self, rect: Rect) {
    self.max_rect = rect;
    self.frame = self.frame_params.build_from_outer(&self.max_rect);
    self.push_rebuild();
  }

  pub fn insert(&mut self, key: u16, view_params: PageViewParams) {
    let rect = self.get_rect_for_key(key);
    let view = view_params.build(&rect);
    if let Some(view_list) = self.map.get_mut(&key) {
      view_list.insert(view);
    } else {
      self.map.insert(key, view.into());
    }
  }

  pub fn remove_list(&mut self, handle: u16) -> bool {
    self.map.remove(&handle).is_some()
  }

  pub fn remove(&mut self, handle: u16) -> bool {
    self.map.remove(&handle).is_some()
  }
}
