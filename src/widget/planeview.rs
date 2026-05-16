// src/planeview.rs

use crate::{
  widget::{Rect, UnitCursor, WeightedCursor},
};
use crossterm::{
  QueueableCommand, 
  style::{SetAttribute, Attribute},
  cursor::{self, MoveTo},
};
use std::{
  io::{self, Write},
};

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
  pub fn resize<C>(&mut self, 
                      cursor: &C, 
                      new_view_start: u16, 
                      new_view_size: u16
                      ) 
  where C: UnitCursor,
  {
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
  pub fn update<C>(&mut self, cursor: &C) -> bool 
  where C: UnitCursor
  {
    let mut scroll    = false;
    let new_line_head = cursor.head();
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
      view_head:  view_start, 
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
  pub fn resize<C>(&mut self, 
                        cursor: &C, 
                        new_view_start: u16, 
                        new_view_size: u16
                        ) 
  where C: UnitCursor<Unit = char>
  {
    let new_line_head   = cursor.head();
    let cursor_position = self.view_head - self.view_start;
    self.view_start     = new_view_start;
    self.view_size      = new_view_size;
    self.weighted_head      = new_line_head;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.weighted_start = 0;
      self.view_head  = self.view_start + u16::try_from(self.weighted_head).unwrap();

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_head  = self.view_start + self.view_size - 1;
      self.weighted_start = self.weighted_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_head  = self.view_start + cursor_position;
      self.weighted_start = self.weighted_head.saturating_sub(usize::from(cursor_position));
    }
  }
  pub fn update<C>(&mut self, cursor: &C) -> bool 
  where C: WeightedCursor
  {
    // move right
    if usize::from(self.view_head) < cursor.weighted_head() {
      let view_delta     = cursor.weighted_range(self.weighted_head, cursor.head());
      let unit_delta     = cursor.head() - self.weighted_head;
      let max_view_delta = usize::from(
        (self.view_start + self.view_size).saturating_sub(self.view_head));

      // scroll right
      if view_delta >= max_view_delta {
        self.view_head  += u16::try_from(view_delta - max_view_delta).unwrap();
        self.weighted_start += unit_delta.saturating_sub(max_view_delta);
        self.weighted_head   = cursor.weighted_head();
        true
      // no scroll
      } else {
        self.view_head += u16::try_from(view_delta).unwrap();
        self.weighted_head  = cursor.weighted_head();
        false
      }
    // move left
    } else if usize::from(self.view_head) > cursor.weighted_head() {
      self.view_head =
        u16::try_from(
        std::cmp::min(
          usize::from(self.view_head), 
          cursor.weighted_len()
          )).unwrap();
//      self.size_head     = std::cmp::min(cursor.units().len(), self.size_head);
      let max_view_delta = usize::from(self.view_head.saturating_sub(self.view_start));
      let view_delta     = cursor.weighted_range(cursor.head(), self.weighted_head);
      // scroll left
      if view_delta > max_view_delta {
        self.weighted_start = self.weighted_start.saturating_sub(view_delta - max_view_delta);
        self.view_head  = self.view_start 
          + u16::try_from(cursor.head() - self.weighted_start).unwrap();
        self.weighted_head  = cursor.weighted_head();
        true
      // no scroll
      } else {
        self.view_head -= u16::try_from(view_delta).unwrap();
        self.weighted_head  = cursor.weighted_head();
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
