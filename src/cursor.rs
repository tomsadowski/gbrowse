// src/cursor.rs

use crate::{
  GetRect, Rect, Pos,
};


#[derive(Clone, Copy, Debug, Default)]
pub struct Cursor {
  head: usize,
  buff: bool,
}

impl std::ops::Deref for Cursor {
  type Target = usize;
  fn deref(&self) -> &Self::Target {&self.head}
}

impl std::ops::DerefMut for Cursor {
  fn deref_mut(&mut self) -> &mut Self::Target {&mut self.head}
}

impl Cursor {
  pub fn editor(mut self) -> Self {
    self.make_editor();
    self
  }

  pub fn make_editor(&mut self) {
    self.buff = true;
  }

  pub fn get_max<T>(&self, vec: &Vec<T>) -> usize {
    match self.buff {
      true  => vec.len(),
      false => vec.len().saturating_sub(1),
    }
  }

  pub fn peek_backward(&self, delta: usize) -> usize {
    if delta > self.head {
      delta - self.head
    } else {0}
  }

  pub fn peek_forward<T>(&self, vec: &Vec<T>, delta: usize) -> usize {
    let max_head = self.get_max(vec);
    if self.head + delta > max_head {
      self.head + delta - max_head
    } else {0}
  }

  pub fn fit<T>(&mut self, vec: &Vec<T>) {
    self.head = self.get_max(vec).min(self.head);
  }

  pub fn move_to_start(&mut self) {
    self.head = 0;
  }

  pub fn move_to_end<T>(&mut self, vec: &Vec<T>) {
    self.head = self.get_max(vec);
  }

  pub fn move_backward(&mut self, mut delta: usize) -> usize {
    if delta > self.head {
      delta -= self.head; 
      self.head = 0; 
      delta
    } else {
      self.head -= delta; 
      0
    }
  }

  pub fn move_forward<T>(&mut self, vec: &Vec<T>, mut delta: usize) -> usize {
    if self.head + delta > self.get_max(vec) {
      delta     = self.head + delta - self.get_max(vec);
      self.head = self.get_max(vec);
      delta
    } else {
      self.head += delta;
      0
    }
  }

  pub fn move_backward_wrapped<T>(&mut self, vec: &Vec<T>, delta: usize) 
    -> bool 
  {
    if vec.len() <= 1 {
      false
    } else if delta > self.head {
      self.move_to_end(vec);
      true
    } else {
      self.head -= delta;
      true
    }
  }

  pub fn move_forward_wrapped<T>(&mut self, vec: &Vec<T>, delta: usize) 
    -> bool 
  {
    if vec.len() <= 1 {
      false
    } else if self.head + delta > self.get_max(vec) {
      self.move_to_start();
      true
    } else {
      self.head += delta;
      true
    }
  }

  pub fn remove<T>(&mut self, vec: &mut Vec<T>) -> usize {
    if self.head < vec.len() {
      vec.remove(self.head);
      self.move_backward_wrapped(vec, 1);
    }
    vec.len()
  }

  pub fn insert_or_move<T, F>(&mut self, vec: &mut Vec<T>, func: F, unit: T) 
    -> bool
  where F: Fn(&T) -> bool,
  {
    if let Some((idx, _)) = vec
      .iter_mut()
      .enumerate()
      .find(|(_, u)| func(u))
    {
      self.head = idx;
      false
    } else if vec.len() == 0 {
      vec.push(unit);
      true
    } else if self.head + 1 == vec.len() {
      vec.push(unit);
      self.head += 1;
      true
    }
    else {
      self.head += 1;
      vec.insert(self.head, unit);
      true
    }
  }

  pub fn delete<T>(&mut self, vec: &mut Vec<T>) -> bool {
    if self.head < vec.len() {
      vec.remove(self.head);
      true
    } else {false}
  }

  pub fn backspace<T>(&mut self, vec: &mut Vec<T>) -> bool {
    if self.peek_backward(1) == 0 {
      self.move_backward(1);
      vec.remove(self.head);
      true
    } else {false}
  }

  pub fn insert<T>(&mut self, vec: &mut Vec<T>, c: T) -> bool {
    if self.head + 1 == vec.len() || vec.len() == 0 {
      vec.push(c);
      self.move_forward(vec, 1);
      true
    } else {
      vec.insert(self.head, c);
      self.move_forward(vec, 1);
      true
    }
  }

  pub fn get_weighted_head(&self, vec: Vec<char>) -> usize {
    use unicode_width::UnicodeWidthChar;
    vec
      .iter()
      .take(self.head)
      .map(|c| c.width().unwrap_or(0))
      .sum()
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
  pub x: Cursor,
  pub y: Cursor,
}

impl Point {
  pub fn editor(mut self) -> Self {
    self.make_editor();
    self
  }

  pub fn make_editor(&mut self) {
    self.x.make_editor();
  }

  pub fn get_linear_head<T>(&self, vec: &Vec<Vec<T>>) -> usize {
    vec
      .iter()
      .take(self.y.saturating_sub(1))
      .map(|v| v.len().max(1))
      .chain(std::iter::once(*self.x))
      .sum()
  }

  pub fn set_linear_head<T>(&mut self, vec: &Vec<Vec<T>>, idx: usize) {
    self.y.move_to_start();
    self.x.move_to_start();
    self.move_right(vec, idx);
  }

  pub fn move_up<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    //let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.y.move_backward(delta) != delta {
      vec.get(*self.y).map(|v| self.x.fit(v));
      true
    } else {false}
  }

  pub fn move_down<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    //let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.y.move_forward(vec, delta) != delta {
      vec.get(*self.y).map(|v| self.x.fit(v));
      true
    } else {false}
  }

  pub fn move_left<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> usize {
    let remainder = self.x.move_backward(delta);
    if remainder != 0 && self.y.move_backward(1) == 0 {
      match vec.get(*self.y).map(|v| self.x.move_to_end(v)) {
        None => remainder,
        _    => self.move_left(vec, remainder.saturating_sub(1)),
      }
    } else {remainder}
  }

  pub fn move_right<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> usize {
    let Some(remainder) = 
      vec.get(*self.y).map(|v| self.x.move_forward(v, delta)) 
      else {return delta};
    if remainder != 0 && self.y.move_forward(vec, 1) == 0 {
      self.x.move_to_start();
      self.move_right(vec, remainder.saturating_sub(1))
    } else {remainder}
  }
}

pub fn get_weighted_length(vec: &Vec<char>) -> usize {
  use unicode_width::UnicodeWidthChar;
  vec.iter().map(|c| c.width().unwrap_or(0)).sum()
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  pos: Pos,
  x: LineCursorView,
  y: LineCursorView,
}

impl<V: GetRect> From<&V> for ScreenCursor {
  fn from(view: &V) -> Self {
    let rect = view.get_rect();
    Self {
      x: LineCursorView::from_size(rect.w),
      y: LineCursorView::from_size(rect.h),
      pos: rect.pos(),
    }
  }
}

impl GetRect for ScreenCursor {
  fn get_rect(&self) -> Rect {
    Rect {
      x: self.pos.x(),
      y: self.pos.y(),
      w: self.x.size,
      h: self.y.size,
    }
  }
}

impl ScreenCursor {
  pub fn get_x_view(&self)   -> LineCursorView {self.x}
  pub fn get_y_view(&self)   -> LineCursorView {self.y}
  pub fn get_x_cursor(&self) -> u16            {
    self.pos.x() + self.x.cursor
  }
  pub fn get_y_cursor(&self) -> u16            {
    self.pos.y() + self.y.cursor
  }
  pub fn get_width(&self)    -> u16            {self.x.size}
  pub fn get_height(&self)   -> u16            {self.y.size}
  pub fn get_x_scroll(&self) -> usize          {self.x.scroll}
  pub fn get_y_scroll(&self) -> usize          {self.y.scroll}

  pub fn resize<V>(&mut self, point: &Point, view: &V)
  where V: GetRect,
  {
    let rect = view.get_rect();
    self.pos = rect.pos();
    self.y.resize(*point.y, rect.h);
    self.x.resize(*point.x, rect.w);
  }

  pub fn update(&mut self, point: &Point) -> bool {
    let y = self.y.update(*point.y);
    let x = self.x.update(*point.x);
    x || y
  }
}

impl crate::Draw for ScreenCursor {
  fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor};
    w
      .queue(cursor::MoveTo(self.get_x_cursor(), self.get_y_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LineCursorView {
  head:   usize, // data head
  scroll: usize, // start of displayable data
  cursor: u16,   // on-screen cursor
  size:   u16,   // width or height of rectangle
}

impl LineCursorView {
  pub fn from_size(size: u16) -> Self {
    Self {
      scroll: 0, 
      head:   0, 
      cursor: 0, 
      size,
    }
  }

  pub fn get_unit_view<T>(self, vec: &Vec<T>) -> Vec<&T> {
    vec
      .iter()
      .skip(self.scroll)
      .take(self.size.into())
      .collect() 
  }

  pub fn get_weighted_view(self, vec: &Vec<char>) -> Vec<&char> {
    use unicode_width::UnicodeWidthChar;
    let size         = usize::from(self.size);  
    let mut text     = vec.iter().skip(self.scroll);
    let mut acc_size = 0;
    let mut result   = vec![];
    while let Some(c) = text.next() && acc_size < size {
      acc_size += c.width().unwrap_or(0);
      result.push(c);
    }
    result
  }

  // preserve cursor position if it still fits in the new bounds
  pub fn resize(
    &mut self, 
    new_head:  usize, 
    new_size:  u16
  ) {
    // position of cursor on screen
    self.size  = new_size;
    self.head  = new_head;
    // go to beginning of line
    if new_head < usize::from(new_size) {
      self.scroll = 0;
      self.cursor = u16::try_from(self.head).unwrap();
    // position must be lowered to fit within new bounds
    } else if self.cursor > new_size - 1 {
      self.cursor = self.size - 1;
      self.scroll = self.head - usize::from(self.size - 1);
    // position can be preserved
    } else {
      self.scroll = self.head.saturating_sub(self.cursor.into());
    }
  }

  pub fn update(&mut self, new_head: usize) -> bool {
    // no move
    if self.head == new_head {
      false
    // move forward
    } else if self.head < new_head {
      let delta_size = new_head - self.head;
      let max_delta  = self.size
        .saturating_sub(1)
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
      // no scroll
      if delta_size <= usize::from(self.cursor) {
        self.cursor -= u16::try_from(delta_size).unwrap();
        self.head    = new_head;
        false
      // scroll backward
      } else { 
        self.scroll = self.scroll.saturating_sub(
          delta_size - usize::from(self.cursor)
        );
        self.cursor = u16::try_from(new_head - self.scroll).unwrap();
        self.head = new_head;
        true
      }
    } 
  }
}
