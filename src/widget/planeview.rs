// src/planeview.rs

use crate::{
  widget::{Rect, UnitCursor},
};
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Debug, Default)]
pub struct ViewCursor {
  pub unit_head:  usize,
  pub unit_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl ViewCursor {
  pub fn new(view_start: u16, view_size: u16) -> Self {
    Self {
      unit_start: 0, 
      unit_head:  0, 
      view_head:  view_start, 
      view_start, 
      view_size
    }
  }
  pub fn scroll(&self) -> usize {
    self.unit_start
  }
  pub fn cursor(&self) -> u16 {
    self.view_head
  }
  // preserve cursor position if it still fits in the new bounds
  pub fn resize_y<Y>(&mut self, 
                      unit_cursor: &Y, 
                      new_view_start: u16, 
                      new_view_size: u16
                      ) 
  where Y: UnitCursor,
  {
    let new_line_head   = unit_cursor.head();
    let cursor_position = self.view_head - self.view_start;
    self.view_start     = new_view_start;
    self.view_size      = new_view_size;
    self.unit_head      = new_line_head;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.unit_start = 0;
      self.view_head  = self.view_start + u16::try_from(self.unit_head).unwrap();

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head  = self.view_start + self.view_size - 1;
      self.unit_start = self.unit_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head  = self.view_start + cursor_position;
      self.unit_start = self.unit_head.saturating_sub(usize::from(cursor_position));
    }
  }
  // preserve cursor position if it still fits in the new bounds
  pub fn resize_x<X, Z>(&mut self, 
                        unit_cursor: &X, 
                        new_view_start: u16, 
                        new_view_size: u16
                        ) 
  where X: UnitCursor<Unit = Z>, Z: UnicodeWidthChar,
  {
    let new_line_head   = unit_cursor.head();
    let cursor_position = self.view_head - self.view_start;
    self.view_start     = new_view_start;
    self.view_size      = new_view_size;
    self.unit_head      = new_line_head;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.unit_start = 0;
      self.view_head  = self.view_start + u16::try_from(self.unit_head).unwrap();

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head  = self.view_start + self.view_size - 1;
      self.unit_start = self.unit_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head  = self.view_start + cursor_position;
      self.unit_start = self.unit_head.saturating_sub(usize::from(cursor_position));
    }
  }
  pub fn yupdate<Y, X>(&mut self, unit_cursor: &Y) -> bool 
  where Y: UnitCursor<Unit = X>
  {
    let mut scroll    = false;
    let new_line_head = unit_cursor.head();
    // forward
    if new_line_head > self.unit_head {
      let diff     = new_line_head - self.unit_head;
      let proposed = usize::from(self.view_head) + diff;
      let max      = usize::from(self.view_start + self.view_size) - 1;
      // scroll forward
      if proposed >= max {
        self.unit_start = self.unit_start + proposed - max;
        scroll          = true;
      }
    // backward
    } else if new_line_head < self.unit_head {
      let diff     = self.unit_head - new_line_head;
      let max_diff = usize::from(self.view_head.saturating_sub(self.view_start));
      // scroll backward
      if diff > max_diff {
        self.unit_start = self.unit_start.saturating_sub(diff - max_diff);
        scroll          = true;
      }
    }
    self.view_head = self.view_start 
      + u16::try_from(new_line_head - self.unit_start).unwrap();
    self.unit_head = new_line_head;
    scroll
  }
  pub fn xupdate<X, Z>(&mut self, cursor: &X) -> bool 
  where X: UnitCursor<Unit = Z>, Z: UnicodeWidthChar + Copy,
  {
    eprintln!("unit_head: {}", self.unit_head);
    // move right
    if self.unit_head < cursor.head() {
      let view_delta = cursor
        .units()[self.unit_head..cursor.head()]
        .iter()
        .fold(0, |acc, u| acc + u.width().unwrap_or(0));

      let unit_delta     = cursor.head() - self.unit_head;
      let max_view_delta = usize::from(
        (self.view_start + self.view_size).saturating_sub(self.view_head));

      // scroll right
      if view_delta >= max_view_delta {
        self.view_head  += u16::try_from(view_delta - max_view_delta).unwrap();
        self.unit_start += unit_delta.saturating_sub(max_view_delta);
        self.unit_head   = cursor.head();
        true
      // no scroll
      } else {
        self.view_head += u16::try_from(view_delta).unwrap();
        self.unit_head = cursor.head();
        false
      }
    // move left
    } else if self.unit_head > cursor.head() {
      self.unit_head = self.unit_head.min(cursor.units().len());
      let max_delta  = usize::from(self.view_head.saturating_sub(self.view_start));
      let view_delta = cursor
        .units()[cursor.head()..self.unit_head]
        .iter()
        .fold(0, |acc, u| acc + u.width().unwrap_or(0));
      // scroll left
      if view_delta > max_delta {
        self.unit_start = self.unit_start.saturating_sub(view_delta - max_delta);
        self.view_head  = self.view_start 
          + u16::try_from(cursor.head() - self.unit_start).unwrap();
        self.unit_head  = cursor.head();
        true
      // no scroll
      } else {
        self.view_head = self.view_start 
          + u16::try_from(cursor.head() - self.unit_start).unwrap();
        self.unit_head = cursor.head();
        false
      }
    // no move
    } else {
      false
    }
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
    self.x.unit_start
  }
  pub fn y_scroll(&self) -> usize {
    self.y.unit_start
  }
  pub fn resize<X, Y, Z>(&mut self, plane: &Y, rect: &Rect) 
  where Y: UnitCursor<Unit = X> , X: UnitCursor<Unit = Z>, Z: UnicodeWidthChar,
  {
    self.y.resize_y(plane, rect.y, rect.h);
    self.x.resize_x(plane.current(), rect.x, rect.w);
  }
  pub fn update<X, Y, Z>(&mut self, plane: &Y) -> bool 
  where Y: UnitCursor<Unit = X> , X: UnitCursor<Unit = Z>, Z: UnicodeWidthChar + Copy,
  {
    let y = self.y.yupdate(plane);
    let x = self.x.xupdate(plane.current());
    x || y
  }
}
