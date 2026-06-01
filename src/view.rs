// src/view.rs

use crate::cursor::{UnitCursor, UnitCursorMut, WeightedCursor};
use crossterm::{
  QueueableCommand, 
  cursor::MoveTo,
};
use std::ops::Range;


pub trait ViewPort {
  fn view_port(&self) -> Rect;
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}
impl ViewPort for Rect {
  fn view_port(&self) -> Rect {
    self.clone()
  }
}
impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    Self {x: 0, y: 0, w, h}
  }
  pub fn crop_north(mut self, delta: u16) -> Self {
    if delta * 2 < self.h {
      self.y += delta;
      self.h -= delta;
    }
    self
  }
  pub fn crop_south(mut self, delta: u16) -> Self {
    if delta < self.h {
      self.h -= delta;
    }
    self
  }
  pub fn crop_east(mut self, delta: u16) -> Self {
    if delta < self.w {
      self.w -= delta
    }
    self
  }
  pub fn crop_west(mut self, delta: u16) -> Self {
    if delta * 2 < self.w {
      self.x += delta;
      self.w -= delta;
    }
    self
  }
  pub fn crop_y(mut self, delta: u16) -> Self {
    self.crop_north(delta).crop_south(delta)
  }
  pub fn crop_x(mut self, delta: u16) -> Self {
    self.crop_east(delta).crop_west(delta)
  }
  pub fn cap_width(mut self, w: u16) -> Self {
    self.w = w.min(self.w);
    self
  }
  pub fn cap_height(mut self, h: u16) -> Self {
    self.h = h.min(self.h);
    self
  }
  pub fn resize(&mut self, w: u16, h: u16) {
    self.w = w; 
    self.h = h;
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
    (self.x_end().saturating_sub(1), self.y_end().saturating_sub(1))
  }
  pub fn row(&self, y: u16) -> Self {
    Self {x: self.x, y: y, w: self.w, h: 1}
  }
  pub fn top_row(&self) -> Self {
    self.row(self.y)
  }
  pub fn bottom_row(&self) -> Self {
    self.row(self.y_end())
  }
  pub fn cropped_north(&self, delta: u16) -> Self {
    self.clone().crop_north(delta)
  }
  pub fn cropped_south(&self, delta: u16) -> Self {
    self.clone().crop_south(delta)
  }
  pub fn cropped_east(&self, delta: u16) -> Self {
    self.clone().crop_east(delta)
  }
  pub fn cropped_west(&self, delta: u16) -> Self {
    self.clone().crop_west(delta)
  }
  pub fn cropped_x(&self, delta: u16) -> Self {
    self.clone().crop_x(delta)
  }
  pub fn cropped_y(&self, delta: u16) -> Self {
    self.clone().crop_y(delta)
  }
  pub fn north_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: self.y, end: rect.y}
  }
  pub fn south_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: rect.y_end(), end: self.y_end()}
  }
  pub fn east_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: rect.x_end(), end: self.x_end()}
  }
  pub fn west_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: self.x, end: rect.x}
  }
  pub fn x_range(&self) -> Range<u16> {
    Range {start: self.x, end: self.x_end()}
  }
  pub fn y_range(&self) -> Range<u16> {
    Range {start: self.y, end: self.y_end()}
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

  pub fn scroll(&self) -> usize {
    self.start
  }

  pub fn cursor(&self) -> u16 {
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
