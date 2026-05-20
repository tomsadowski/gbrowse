// src/cursor.rs

use crate::rect::Rect;
use unicode_width::UnicodeWidthChar;
use crossterm::{QueueableCommand, cursor::{self, MoveTo}};
use std::{iter::Take, slice, io::{self, Write}};

pub trait UnitCursor {
  type Unit;
  fn units(&self)        -> &Vec<Self::Unit>;
  fn head(&self)         -> usize;
  fn head_mut(&mut self) -> &mut usize;
  fn max_head(&self)     -> usize;

  fn current(&self) -> &Self::Unit {
    &self.units()[self.head()]
  }
  fn fit(&mut self, new_cursor: usize) {
    *self.head_mut() = self.max_head().min(new_cursor);
  }
  fn start(&mut self) {
    *self.head_mut() = 0;
  }
  fn end(&mut self) {
    *self.head_mut() = self.max_head();
  }
  fn view_units(&self, start: usize, width: usize) -> Take<slice::Iter<'_, Self::Unit>> {
    if start >= self.units().len() {
      self.units().iter().take(0) 
    } else {
      self.units()[start..].iter().take(width) 
    }
  }
  fn peek_backward(&self, delta: usize) -> usize {
    if delta > self.head() {
      delta - self.head()
    } else {0}
  }
  fn peek_forward(&self, delta: usize) -> usize {
    let max_head = self.max_head();
    if self.head() + delta > max_head {
      self.head() + delta - max_head
    } else {0}
  }
  fn backward(&mut self, mut delta: usize) -> usize {
    if delta > self.head() {
      delta -= self.head();
      *self.head_mut() = 0;
      delta
    } else {
      *self.head_mut() -= delta;
      0
    }
  }
  fn forward(&mut self, mut delta: usize) -> usize {
    if self.head() + delta > self.max_head() {
      delta = self.head() + delta - self.max_head();
      *self.head_mut() = self.max_head();
      delta
    } else {
      *self.head_mut() += delta;
      0
    }
  }
  fn wrapping_backward(&mut self, delta: usize) {
    if delta > self.head() {
      self.end();
    } else {
      *self.head_mut() -= delta;
    }
  }
  fn wrapping_forward(&mut self, delta: usize) {
    if self.head() + delta > self.max_head() {
      self.start();
    } else {
      *self.head_mut() += delta;
    }
  }
}

pub trait UnitCursorMut: UnitCursor {
  fn units_mut(&mut self) -> &mut Vec<Self::Unit>;

  fn delete(&mut self) -> bool {
    let head = self.head();
    if head < self.units().len() {
      self.units_mut().remove(head);
      true
    } else {false}
  }
  fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.backward(1);
      let head = self.head();
      self.units_mut().remove(head);
      true
    } else {false}
  }
  fn insert(&mut self, c: Self::Unit) -> bool {
    let head = self.head();
    if head + 1 == self.units().len() || self.units().len() == 0 {
      self.units_mut().push(c);
      self.forward(1);
      true
    } else {
      self.units_mut().insert(head, c);
      self.forward(1);
      true
    }
  }
}

pub trait WeightedCursor: UnitCursor {
  fn weighted_head(&self) -> usize;
  fn weighted_len(&self) -> usize;
  fn view_weighted(&self, start: usize, width: usize) -> Take<slice::Iter<'_, Self::Unit>>;
}
impl<U> WeightedCursor for U where U: UnitCursor<Unit = char> {
  fn weighted_head(&self) -> usize {
    self.units()[..self.head()].iter().fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }
  fn weighted_len(&self) -> usize {
    self.units().iter().fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }
  fn view_weighted(&self, start: usize, width: usize) -> Take<slice::Iter<'_, Self::Unit>> {
    if start >= self.units().len() {
      self.units().iter().take(0) 
    } else {
      let text           = &self.units()[start..];
      let mut acc_width  = 0;
      let mut unit_count = 0;
      while acc_width < width && unit_count < text.len() {
        acc_width  += &text[unit_count].width().unwrap_or(0);
        unit_count += 1;
      }
      text.iter().take(unit_count)
    }
  }
}

#[derive(Clone, Debug, Default)]
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
      start: 0, 
      head:  0, 
      view_head:      view_start, 
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
  pub fn resize(&mut self, new_head: usize, new_view_start: u16, new_view_size: u16) {
    let cursor_position   = self.view_head - self.view_start;
    self.view_start       = new_view_start;
    self.view_size        = new_view_size;
    self.head    = new_head;
    // go to beginning of line
    if new_head < usize::from(new_view_size) {
      self.start = 0;
      self.view_head      = self.view_start + u16::try_from(self.head).unwrap();
    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head      = self.view_start + self.view_size - 1;
      self.start = self.head - usize::from(self.view_size - 1);
    // cursor_position can be preserved
    } else {
      self.view_head      = self.view_start + cursor_position;
      self.start = self.head.saturating_sub(usize::from(cursor_position));
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

#[derive(Clone, Debug, Default)]
pub struct ScreenCursor {
  pub x: CursorView,
  pub y: CursorView,
}
impl ScreenCursor {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: CursorView::new(rect.x, rect.w),
      y: CursorView::new(rect.y, rect.h),
    }
  }
  pub fn x_cursor(&self) -> u16 {
    self.x.view_head
  }
  pub fn y_cursor(&self) -> u16 {
    self.y.view_head
  }
  pub fn x_scroll(&self) -> usize {
    self.x.start
  }
  pub fn y_scroll(&self) -> usize {
    self.y.start
  }
  pub fn resize<X, Y>(&mut self, plane: &Y, rect: &Rect) 
  where Y: UnitCursor<Unit = X> , X: WeightedCursor 
  {
    self.y.resize(plane.head(), rect.y, rect.h);
    self.x.resize(plane.current().weighted_head(), rect.x, rect.w);
  }
  pub fn update<X, Y>(&mut self, plane: &Y) -> bool 
  where Y: UnitCursor<Unit = X> , X: WeightedCursor
  {
    let y = self.y.update(plane.head());
    let x = self.x.update(plane.current().weighted_head());
    x || y
  }
  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer
      .queue(MoveTo(self.x.cursor(), self.y.cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}
