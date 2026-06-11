// src/screencursor.rs

use crate::{
  UnitCursor, 
  WeightedCursor,
  ViewPort,
  Rect,
};
use std::io::Write;


#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  x: LineCursor,
  y: LineCursor,
}

impl<V: ViewPort> From<&V> for ScreenCursor {
  fn from(view: &V) -> Self {
    let view = view.get_view_port();
    Self {
      x: LineCursor::new(view.x, view.w),
      y: LineCursor::new(view.y, view.h),
    }
  }
}

impl ViewPort for ScreenCursor {
  fn get_view_port(&self) -> Rect {
    Rect {
      x: self.x.get_start(),
      y: self.y.get_start(),
      w: self.x.get_size(),
      h: self.y.get_size(),
    }
  }
}

impl ScreenCursor {
  pub fn get_x_line(&self) -> LineCursor {
    self.x
  }

  pub fn get_y_line(&self) -> LineCursor {
    self.y
  }

  pub fn get_x_cursor(&self) -> u16 {
    self.x.get_cursor()
  }

  pub fn get_y_cursor(&self) -> u16 {
    self.y.get_cursor()
  }

  pub fn get_width(&self) -> u16 {
    self.x.get_size()
  }

  pub fn get_height(&self) -> u16 {
    self.y.get_size()
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
pub struct LineCursor {
  /// data head
  head:    usize,
  /// start of displayable data
  scroll:  usize,
  /// on-screen cursor
  cursor:  u16,
  /// x or y
  start:   u16,
  /// width or height of rectangle
  size:    u16,
}

impl LineCursor {
  pub fn new(start: u16, size: u16) -> Self {
    Self {
      scroll:     0, 
      head:       0, 
      cursor: start, 
      start, 
      size,
    }
  }

  pub fn get_scroll(&self) -> usize {self.scroll}

  pub fn get_cursor(&self) -> u16 {self.cursor}

  pub fn get_size(&self)   -> u16 {self.size}

  pub fn get_start(&self)  -> u16 {self.start}

  // preserve cursor position if it still fits in the new bounds
  pub fn resize(
    &mut self, 
    new_head:  usize, 
    new_start: u16, 
    new_size:  u16
  ) {
    // position of cursor on screen
    let position = self.cursor - self.start;
    self.start = new_start;
    self.size  = new_size;
    self.head  = new_head;
    // go to beginning of line
    if new_head < usize::from(new_size) {
      self.scroll = 0;
      self.cursor = self.start + u16::try_from(self.head).unwrap();
    // position must be lowered to fit within new bounds
    } else if position > new_size - 1 {
      self.cursor = self.start + self.size - 1;
      self.scroll = self.head - usize::from(self.size - 1);
    // position can be preserved
    } else {
      self.cursor = self.start + position;
      self.scroll = self.head.saturating_sub(usize::from(position));
    }
  }

  pub fn update(&mut self, new_head: usize) -> bool {
    // no move
    if self.head == new_head {
      false
    // move forward
    } else if self.head < new_head {
      let delta_size = new_head - self.head;
      let max_delta  = (self.start + self.size.saturating_sub(1))
          .saturating_sub(self.cursor);
      // no scroll
      if delta_size < usize::from(max_delta) { 
        self.cursor  += u16::try_from(delta_size).unwrap();
        self.head     = new_head;
        false
      // scroll forward
      } else {
        self.scroll += delta_size - usize::from(max_delta);
        self.cursor += max_delta;
        self.head    = new_head;
        true
      }
    // move backward
    } else { 
      let delta_size = self.head - new_head;
      let max_delta  = self.cursor.saturating_sub(self.start);
      // no scroll
      if delta_size <= usize::from(max_delta) {
        self.cursor -= u16::try_from(delta_size).unwrap();
        self.head    = new_head;
        false
      // scroll backward
      } else { 
        self.scroll = self.scroll.saturating_sub(
          delta_size - usize::from(max_delta)
        );
        self.cursor = self.start + 
          u16::try_from(new_head - self.scroll).unwrap();
        self.head = new_head;
        true
      }
    } 
  }
}
