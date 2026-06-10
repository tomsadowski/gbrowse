// src/view.rs

use crate::{
  UnitCursor, 
  WeightedCursor,
  ViewPort,
  Rect,
};
use std::io::Write;


#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  x: CursorView,
  y: CursorView,
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
    Y: UnitCursor<Unit = X>, 
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

#[derive(Copy, Clone, Debug, Default)]
struct CursorView {
  head:  usize,
  start: usize,
  view_head:  u16,
  view_start: u16,
  view_size:  u16,
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
