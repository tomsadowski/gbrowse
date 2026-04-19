// src/screen.rs

use crate::text::Planar;
use std::ops::Range;

pub fn safe_range<T>(a: T, b: T) -> Range<T> 
where T: PartialOrd + PartialEq
{
  if a <= b {
    Range {start: a, end: b}
  } else {
    Range {start: b, end: a}
  }
}
#[derive(Clone, Default, Copy)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}
impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    Self {x: 0, y: 0, w, h}
  }
  pub fn x_end(&self) -> u16 {
    self.x + self.w
  }
  pub fn y_end(&self) -> u16 {
    self.y + self.h
  }
  pub fn row(&self, y: u16) -> Self {
    Self {x: self.x, y: y, w: self.w, h: 1}
  }
  pub fn a(&self) -> (u16, u16) {
    (self.x, self.y)
  }
  pub fn b(&self) -> (u16, u16) {
    (self.x_end().saturating_sub(1), self.y)
  }
  pub fn c(&self) -> (u16, u16) {
    (self.x, self.y_end().saturating_sub(1))
  }
  pub fn d(&self) -> (u16, u16) {
    (self.x_end().saturating_sub(1), self.y_end().saturating_sub(1))
  }
  pub fn bottom_row(&self) -> Self {
    self.row(self.y_end())
  }
  pub fn x_range(&self) -> Range<u16> {
    Range {start: self.x, end: self.x_end()}
  }
  pub fn y_range(&self) -> Range<u16> {
    Range {start: self.y, end: self.y_end()}
  }
  pub fn limit_h(&self, h: u16) -> Self {
    let mut rect = self.clone();
    rect.h = h.min(self.h);
    rect
  }
  pub fn cropped_west_range(&self, step: u16) -> Range<u16> {
    if self.w >= step {
      Range {start: self.x + step, end: self.x_end()}
    } else {
      Range {start: self.x_end(), end: self.x_end()}
    }
  }
  pub fn cropped_north_range(&self, step: u16) -> Range<u16> {
    if self.h >= step {
      Range {start: self.y + step, end: self.y_end()}
    } else {
      Range {start: self.y_end(), end: self.y_end()}
    }
  }
  pub fn cropped_x_range(&self, step: u16) -> Range<u16> {
    Range {start: self.x + step, end: self.x_end() - step}
  }
  pub fn cropped_y_range(&self, step: u16) -> Range<u16> {
    Range {start: self.y + step, end: self.y_end() - step}
  }
  pub fn cropped_x_points(&self, step: u16) -> (u16, u16) {
    (self.x + step, self.x_end() - (step + 1))
  }
  pub fn resize(&mut self, w: u16, h: u16) {
    self.w = w; 
    self.h = h;
  }
  pub fn cropped_south(mut self, step: u16) -> Self {
    self.clone().crop_south(step)
  }
  pub fn cropped_east(mut self, step: u16) -> Self {
    self.clone().crop_east(step)
  }
  pub fn cropped_north(mut self, step: u16) -> Self {
    self.clone().crop_north(step)
  }
  pub fn cropped_west(mut self, step: u16) -> Self {
    self.clone().crop_west(step)
  }
  pub fn crop_south(mut self, step: u16) -> Self {
    if step < self.h {
      self.h -= step;
    }
    self
  }
  pub fn crop_east(mut self, step: u16) -> Self {
    if step < self.w {
      self.w -= step
    }
    self
  }
  pub fn crop_north(mut self, step: u16) -> Self {
    if step * 2 < self.h {
      self.y += step;
      self.h -= step;
    }
    self
  }
  pub fn crop_west(mut self, step: u16) -> Self {
    if step * 2 < self.w {
      self.x += step;
      self.w -= step;
    }
    self
  }
  pub fn crop_y(self, step: u16) -> Self {
    self.crop_north(step).crop_south(step)
  }
  pub fn crop_x(self, step: u16) -> Self {
    self.crop_east(step).crop_west(step)
  }
  pub fn south_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.y_end();
    let b = rect.y_end();
    safe_range(a, b)
  }
  pub fn east_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.x_end();
    let b = rect.x_end();
    safe_range(a, b)
  }
  pub fn north_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.y;
    let b = rect.y;
    safe_range(a, b)
  }
  pub fn west_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.x;
    let b = rect.x;
    safe_range(a, b)
  }
}

#[derive(Clone, Debug, Default)]
pub struct PlaneView {
  x: LineView,
  y: LineView,
}
impl PlaneView {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: LineView::new(rect.x, rect.w),
      y: LineView::new(rect.y, rect.h),
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
  pub fn resize<P: Planar>(&mut self, plane: &P, rect: &Rect) {
    self.x.resize(plane.x_head(), rect.x, rect.w);
    self.y.resize(plane.y_head(), rect.y, rect.h);
  }
  pub fn update<P: Planar>(&mut self, plane: &P) -> bool {
    let x = self.x.update(plane.x_head());
    let y = self.y.update(plane.y_head());
    x || y
  }
}
#[derive(Clone, Debug, Default)]
pub struct LineView {
  pub line_head:  usize,
  pub line_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl LineView {
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
  pub fn resize(&mut self, 
                new_line_head:  usize, 
                new_view_start: u16, 
                new_view_size:  u16) 
  {
    let cursor_position = self.view_head - self.view_start;
    self.view_start = new_view_start;
    self.view_size = new_view_size;
    self.line_head = new_line_head;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.line_start = 0;
      self.view_head = self.view_start + u16::try_from(self.line_head)
          .expect("We do not have Allah's permission");

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head = self.view_start + self.view_size - 1;
      self.line_start = self.line_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head = self.view_start + cursor_position;
      self.line_start = self.line_head
        .saturating_sub(usize::from(cursor_position));
    }
  }
  pub fn update(&mut self, new_line_head: usize) -> bool {
    let mut scroll = false;
    // forward
    if new_line_head > self.line_head {
      let diff = new_line_head - self.line_head;
      let proposed = usize::from(self.view_head) + diff;
      let max = 
        usize::from(self.view_start) + 
        usize::from(self.view_size) - 1;
      // scroll forward
      if proposed >= max {
        self.line_start = self.line_start + proposed - max;
        scroll = true;
      }
    // backward
    } else if new_line_head < self.line_head {
      let diff = self.line_head - new_line_head;
      let max_diff = 
        usize::from(self.view_head.saturating_sub(self.view_start));
      // scroll backward
      if diff > max_diff {
        self.line_start = self.line_start.saturating_sub(diff - max_diff);
        scroll = true;
      }
    }
    self.view_head = self.view_start + 
      u16::try_from(new_line_head - self.line_start)
        .expect("We do not have Allah's permission");
    self.line_head = new_line_head;
    scroll
  }
}
