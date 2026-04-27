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
  pub fn crop_north(mut self, step: u16) -> Self {
    if step * 2 < self.h {
      self.y += step;
      self.h -= step;
    }
    self
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
  pub fn crop_west(mut self, step: u16) -> Self {
    if step * 2 < self.w {
      self.x += step;
      self.w -= step;
    }
    self
  }
  pub fn crop_y(mut self, step: u16) -> Self {
    self.crop_north(step).crop_south(step)
  }
  pub fn crop_x(mut self, step: u16) -> Self {
    self.crop_east(step).crop_west(step)
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
  pub fn cropped_south(&self, step: u16) -> Self {
    self.clone().crop_south(step)
  }
  pub fn cropped_east(&self, step: u16) -> Self {
    self.clone().crop_east(step)
  }
  pub fn cropped_north(&self, step: u16) -> Self {
    self.clone().crop_north(step)
  }
  pub fn cropped_west(&self, step: u16) -> Self {
    self.clone().crop_west(step)
  }
  pub fn cropped_x(&self, step: u16) -> Self {
    self.clone().crop_south(step).crop_north(step)
  }
  pub fn cropped_y(&self, step: u16) -> Self {
    self.clone().crop_west(step).crop_east(step)
  }
  pub fn south_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: rect.y_end(), end: self.y_end()}
  }
  pub fn east_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: rect.x_end(), end: self.x_end()}
  }
  pub fn north_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: self.y, end: rect.y}
  }
  pub fn west_range(&self, rect: &Rect) -> Range<u16> {
    Range {start: self.x, end: rect.x}
  }
}
