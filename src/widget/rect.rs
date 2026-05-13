// src/rect.rs

use std::ops::Range;

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
