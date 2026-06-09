// src/view.rs

use crate::{
  UnitCursor, 
  WeightedCursor,
};
use std::{
  ops::Range,
  io::Write,
};


pub trait ViewPort {
  fn get_view_port(&self) -> Rect;
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}
impl ViewPort for Rect {
  fn get_view_port(&self) -> Rect {
    self.clone()
  }
}
impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    Self {x: 0, y: 0, w, h}
  }

  pub fn crop_north(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta * 2 < rect.h {
      rect.y += delta;
      rect.h -= delta;
    }
    rect
  }

  pub fn crop_south(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta < rect.h {
      rect.h -= delta;
    }
    rect
  }

  pub fn crop_east(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta < rect.w {
      rect.w -= delta
    }
    rect
  }

  pub fn crop_west(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta * 2 < rect.w {
      rect.x += delta;
      rect.w -= delta;
    }
    rect
  }

  pub fn crop_y(&self, delta: u16) -> Self {
    self
      .crop_north(delta)
      .crop_south(delta)
  }

  pub fn crop_x(&self, delta: u16) -> Self {
    self
      .crop_east(delta)
      .crop_west(delta)
  }

  pub fn row(&self, y: u16) -> Self {
    Self {
      x: self.x, 
      y: y, 
      w: self.w, 
      h: 1
    }
  }

  pub fn top_row(&self) -> Self {
    self.row(self.y)
  }

  pub fn bottom_row(&self) -> Self {
    self.row(self.y_end())
  }

  pub fn cap_width(&self, w: u16) -> Self {
    let mut rect = self.clone();
    rect.w = w.min(rect.w);
    rect
  }

  pub fn cap_height(&self, h: u16) -> Self {
    let mut rect = self.clone();
    rect.h = h.min(rect.h);
    rect
  }

  pub fn x_end(&self) -> u16 {
    self.x + self.w
  }

  pub fn y_end(&self) -> u16 {
    self.y + self.h
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
    (self.x_end().saturating_sub(1), 
     self.y_end().saturating_sub(1))
  }

  pub fn x_range(&self) -> Range<u16> {
    Range {
      start: self.x, 
      end:   self.x_end()
    }
  }

  pub fn y_range(&self) -> Range<u16> {
    Range {
      start: self.y, 
      end:   self.y_end()
    }
  }

  pub fn resize(&mut self, w: u16, h: u16) {
    self.w = w; 
    self.h = h;
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CursorView {
  pub head:  usize,
  pub start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl CursorView {
  pub fn new(view_start: u16, view_size: u16) -> Self {
    Self {
      start:     0, 
      head:      0, 
      view_head: view_start, 
      view_start, 
      view_size
    }
  }

  pub fn get_scroll(&self) -> usize {
    self.start
  }

  pub fn get_cursor(&self) -> u16 {
    self.view_head
  }

  // preserve cursor position if it still fits in the new bounds
  pub fn resize(
    &mut self, 
    new_head: usize, 
    new_view_start: u16, 
    new_view_size: u16
  ) {
    let cursor_position   = self.view_head - self.view_start;
    self.view_start       = new_view_start;
    self.view_size        = new_view_size;
    self.head             = new_head;
    // go to beginning of line
    if new_head < usize::from(new_view_size) {
      self.start     = 0;
      self.view_head = self.view_start + u16::try_from(self.head).unwrap();
    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head = self.view_start + self.view_size - 1;
      self.start     = self.head - usize::from(self.view_size - 1);
    // cursor_position can be preserved
    } else {
      self.view_head = self.view_start + cursor_position;
      self.start     = self.head.saturating_sub(usize::from(cursor_position));
    }
  }

  pub fn update(&mut self, new_head: usize) -> bool {
    // no move
    if self.head == new_head {
      false
    // move forward
    } else if self.head < new_head {
      let delta_size     = new_head - self.head;
      let max_view_delta = 
        (self.view_start + self.view_size.saturating_sub(1))
          .saturating_sub(self.view_head);
      // no scroll
      if delta_size < usize::from(max_view_delta) { 
        self.view_head  += u16::try_from(delta_size).unwrap();
        self.head        = new_head;
        false
      // scroll forward
      } else {
        self.start     += delta_size - usize::from(max_view_delta);
        self.view_head += max_view_delta;
        self.head       = new_head;
        true
      }
    // move backward
    } else { 
      let delta_size     = self.head - new_head;
      let max_view_delta = self.view_head.saturating_sub(self.view_start);
      // no scroll
      if delta_size <= usize::from(max_view_delta) {
        self.view_head -= u16::try_from(delta_size).unwrap();
        self.head       = new_head;
        false
      // scroll backward
      } else { 
        self.start = self.start
          .saturating_sub(delta_size - usize::from(max_view_delta));
        self.view_head = self.view_start 
          + u16::try_from(new_head - self.start).unwrap();
        self.head = new_head;
        true
      }
    } 
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  pub x: CursorView,
  pub y: CursorView,
}
impl<V: ViewPort> From<&V> for ScreenCursor {
  fn from(view: &V) -> Self {
    let view = view.get_view_port();
    Self {
      x: CursorView::new(view.x, view.w),
      y: CursorView::new(view.y, view.h),
    }
  }
}
impl ScreenCursor {
  pub fn get_x_cursor(&self) -> u16 {
    self.x.get_cursor()
  }

  pub fn get_y_cursor(&self) -> u16 {
    self.y.get_cursor()
  }

  pub fn get_x_scroll(&self) -> usize {
    self.x.get_scroll()
  }

  pub fn get_y_scroll(&self) -> usize {
    self.y.get_scroll()
  }

  pub fn resize<X, Y>(&mut self, plane: &Y, rect: &Rect) 
  where 
    Y: UnitCursor<Unit = X> , 
    X: WeightedCursor 
  {
    self.y.resize(plane.get_head(), rect.y, rect.h);
    self.x.resize(
      plane.use_current(|c| c.get_weighted_head()).unwrap_or(0), 
      rect.x, 
      rect.w
    );
  }

  pub fn update<X, Y>(&mut self, plane: &Y) -> bool 
  where 
    Y: UnitCursor<Unit = X>, 
    X: WeightedCursor
  {
    let y = self.y.update(plane.get_head());
    let x = self.x.update(
      plane.use_current(|c| c.get_weighted_head()).unwrap_or(0)
    );
    x || y
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor};
    writer
      .queue(cursor::MoveTo(self.x.get_cursor(), self.y.get_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

