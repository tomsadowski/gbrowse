// src/planeview.rs

use crate::{
  widget::{Rect, DataCursor},
};

#[derive(Clone, Debug, Default)]
pub struct ViewCursor {
  pub line_head:  usize,
  pub line_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl ViewCursor {
  pub fn new(view_start: u16, view_size: u16) -> Self {
    Self {
      line_start: 0, 
      line_head:  0, 
      view_head:  view_start, 
      view_start, 
      view_size
    }
  }
  pub fn scroll(&self) -> usize {
    self.line_start
  }
  pub fn cursor(&self) -> u16 {
    self.view_head
  }
  // preserve cursor position if it still fits in the new bounds
  pub fn resize(&mut self, new_line_head: usize, new_view_start: u16, new_view_size: u16) {
    let cursor_position = self.view_head - self.view_start;
    self.view_start = new_view_start;
    self.view_size  = new_view_size;
    self.line_head  = new_line_head;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.line_start = 0;
      self.view_head  = self.view_start + u16::try_from(self.line_head).unwrap();

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head  = self.view_start + self.view_size - 1;
      self.line_start = self.line_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head  = self.view_start + cursor_position;
      self.line_start = self.line_head.saturating_sub(usize::from(cursor_position));
    }
  }
  pub fn update(&mut self, new_line_head: usize) -> bool {
    let mut scroll = false;
    // forward
    if new_line_head > self.line_head {
      let diff     = new_line_head - self.line_head;
      let proposed = usize::from(self.view_head) + diff;
      let max      = usize::from(self.view_start + self.view_size) - 1;
      // scroll forward
      if proposed >= max {
        self.line_start = self.line_start + proposed - max;
        scroll = true;
      }
    // backward
    } else if new_line_head < self.line_head {
      let diff     = self.line_head - new_line_head;
      let max_diff = usize::from(self.view_head.saturating_sub(self.view_start));
      // scroll backward
      if diff > max_diff {
        self.line_start = self.line_start.saturating_sub(diff - max_diff);
        scroll = true;
      }
    }
    self.view_head = self.view_start + u16::try_from(new_line_head - self.line_start).unwrap();
    self.line_head = new_line_head;
    scroll
  }
}

#[derive(Clone, Debug, Default)]
pub struct ScreenCursor {
  x: ViewCursor,
  y: ViewCursor,
}
impl ScreenCursor {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: ViewCursor::new(rect.x, rect.w),
      y: ViewCursor::new(rect.y, rect.h),
    }
  }
  pub fn x_cursor(&self) -> u16 {
    self.x.view_head
  }
  pub fn y_cursor(&self) -> u16 {
    self.y.view_head
  }
  pub fn x_scroll(&self) -> usize {
    self.x.line_start
  }
  pub fn y_scroll(&self) -> usize {
    self.y.line_start
  }
  pub fn resize<X, Y, Z>(&mut self, plane: &Y, rect: &Rect) 
  where Y: DataCursor<X>, X: DataCursor<Z>
  {
    self.y.resize(plane.head(), rect.y, rect.h);
    self.x.resize(plane.current().head(), rect.x, rect.w);
  }
  pub fn update<X, Y, Z>(&mut self, plane: &Y) -> bool 
  where Y: DataCursor<X>, X: DataCursor<Z>
  {
    let y = self.y.update(plane.head());
    let x = self.x.update(plane.current().head());
    x || y
  }
}
