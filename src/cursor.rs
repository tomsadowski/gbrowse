// src/cursor.rs

use crate::{
  rect::Rect,
};
use crossterm::{
  QueueableCommand, 
  cursor::{self, MoveTo},
};
use unicode_width::UnicodeWidthChar;
use std::io::{self, Write};

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

  fn view_units(&self, start: usize, width: usize) -> std::iter::Take<std::slice::Iter<'_, Self::Unit>> {
    let start = std::cmp::min(start, self.units().len().saturating_sub(1));
    self.units()[start..].iter().take(width) 
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
  fn view_weighted(&self, start: usize, width: usize) -> std::iter::Take<std::slice::Iter<'_, Self::Unit>>;
}
impl<U> WeightedCursor for U where U: UnitCursor<Unit = char> {
  fn weighted_head(&self) -> usize {
    self.units()[..self.head()].iter().fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }

  fn weighted_len(&self) -> usize {
    self.units().iter().fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }

  fn view_weighted(&self, start: usize, width: usize) -> std::iter::Take<std::slice::Iter<'_, Self::Unit>> {
    let start         = std::cmp::min(start, self.units().len().saturating_sub(1));
    let text          = &self.units()[start..];
    let mut w         = 0;
    let mut max_width = 0;
    while w < width && max_width < text.len() {
      w         += &text[max_width].width().unwrap_or(0);
      max_width += 1;
    }
    self.units()[start..].iter().take(max_width)
  }
}

#[derive(Clone, Debug, Default)]
pub struct UnitCursorView {
  pub unit_head:  usize,
  pub unit_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl UnitCursorView {
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
  pub fn resize<C: UnitCursor>(&mut self, cursor: &C, new_view_start: u16, new_view_size: u16) {
    let new_line_head   = cursor.head();
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
  pub fn update<C: UnitCursor>(&mut self, cursor: &C) -> bool {
    let mut scroll    = false;
    let new_head = cursor.head();

    // forward
    if new_head > self.unit_head {
      let diff     = new_head - self.unit_head;
      let proposed = usize::from(self.view_head) + diff;
      let max      = usize::from(self.view_start + self.view_size) - 1;

      // scroll forward
      if proposed >= max {
        self.unit_start = self.unit_start + proposed - max;
        scroll          = true;
      }

    // backward
    } else if new_head < self.unit_head {
      let diff     = self.unit_head - new_head;
      let max_diff = usize::from(self.view_head.saturating_sub(self.view_start));

      // scroll backward
      if diff > max_diff {
        self.unit_start = self.unit_start.saturating_sub(diff - max_diff);
        scroll          = true;
      }
    }
    self.view_head = self.view_start + u16::try_from(new_head - self.unit_start).unwrap();
    self.unit_head = new_head;
    scroll
  }
}

#[derive(Clone, Debug, Default)]
pub struct WeightedCursorView {
  pub weighted_head:  usize,
  pub weighted_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl WeightedCursorView {
  pub fn new(view_start: u16, view_size: u16) -> Self {
    Self {
      weighted_start: 0, 
      weighted_head:  0, 
      view_head:      view_start, 
      view_start, 
      view_size
    }
  }

  pub fn scroll(&self) -> usize {
    self.weighted_start
  }

  pub fn cursor(&self) -> u16 {
    self.view_head
  }

  // preserve cursor position if it still fits in the new bounds
  pub fn resize<C: WeightedCursor>(&mut self, cursor: &C, new_view_start: u16, new_view_size: u16) {
    let new_weighted_head = cursor.weighted_head();
    let cursor_position   = self.view_head - self.view_start;
    self.view_start       = new_view_start;
    self.view_size        = new_view_size;
    self.weighted_head    = new_weighted_head;

    // go to beginning of line
    if new_weighted_head < usize::from(new_view_size) {
      self.weighted_start = 0;
      self.view_head      = self.view_start + u16::try_from(self.weighted_head).unwrap();

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head      = self.view_start + self.view_size - 1;
      self.weighted_start = self.weighted_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head      = self.view_start + cursor_position;
      self.weighted_start = self.weighted_head.saturating_sub(usize::from(cursor_position));
    }
  }

  pub fn update<C: WeightedCursor>(&mut self, cursor: &C) -> bool {
    let new_weighted_head = cursor.weighted_head();

    // no move
    if self.weighted_head == new_weighted_head {
      false

    // move forward
    } else if self.weighted_head < new_weighted_head {
      let delta_size     = new_weighted_head - self.weighted_head;
      let max_view_delta = (self.view_start + self.view_size).saturating_sub(self.view_head);

      // no scroll
      if delta_size <= usize::from(max_view_delta) { 
        self.view_head     += u16::try_from(delta_size).unwrap();
        self.weighted_head  = new_weighted_head;
        false

      // scroll forward
      } else {
        self.weighted_start += delta_size - usize::from(max_view_delta);
        self.view_head      += max_view_delta;
        self.weighted_head   = new_weighted_head;
        true
      }

    // move backward
    } else { 
      let delta_size     = self.weighted_head - new_weighted_head;
      let max_view_delta = self.view_head.saturating_sub(self.view_start);

      // no scroll
      if delta_size <= usize::from(max_view_delta) {
        self.view_head     -= u16::try_from(delta_size).unwrap();
        self.weighted_head  = new_weighted_head;
        false

      // scroll backward
      } else { 
        self.weighted_start = self.weighted_start.saturating_sub(delta_size - usize::from(max_view_delta));
        self.view_head      = self.view_start + u16::try_from(new_weighted_head - self.weighted_start).unwrap();
        self.weighted_head  = new_weighted_head;
        true
      }
    } 
  }
}

#[derive(Clone, Debug, Default)]
pub struct ScreenCursor {
  pub x: WeightedCursorView,
  pub y: UnitCursorView,
}
impl ScreenCursor {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: WeightedCursorView::new(rect.x, rect.w),
      y: UnitCursorView::new(rect.y, rect.h),
    }
  }

  pub fn x_cursor(&self) -> u16 {
    self.x.view_head
  }

  pub fn y_cursor(&self) -> u16 {
    self.y.view_head
  }

  pub fn x_scroll(&self) -> usize {
    self.x.weighted_start
  }

  pub fn y_scroll(&self) -> usize {
    self.y.unit_start
  }

  pub fn resize<X, Y>(&mut self, plane: &Y, rect: &Rect) 
  where Y: UnitCursor<Unit = X> , X: UnitCursor<Unit = char>
  {
    self.y.resize(plane, rect.y, rect.h);
    self.x.resize(plane.current(), rect.x, rect.w);
  }

  pub fn update<X, Y>(&mut self, plane: &Y) -> bool 
  where Y: UnitCursor<Unit = X> , X: WeightedCursor
  {
    let y = self.y.update(plane);
    let x = self.x.update(plane.current());
    x || y
  }

  pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer
      .queue(MoveTo(self.x.cursor(), self.y.cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}
