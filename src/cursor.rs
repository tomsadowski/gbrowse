// src/cursor.rs

use crate::{Rect, Pos, Dim, GetMaxHeight};


#[derive(Clone, Copy, Debug, Default)]
pub struct Cursor {
  pub head: usize,
  pub buff: bool,
}


impl Cursor {
  pub fn editor<T>(mut self, vec: &Vec<T>) -> Self {
    self.buff = true;
    self.move_to_end(vec); self
  }


  pub fn get_max<T>(&self, vec: &Vec<T>) -> usize {
    if self.buff { vec.len() } 
    else { vec.len().saturating_sub(1) }
  }


  pub fn peek_move<T>(&self, vec: &Vec<T>, delta: isize) -> isize {
    self.clone().move_head(vec, delta)
  }


  pub fn fit<T>(&mut self, vec: &Vec<T>, new_head: usize) {
    self.head = self.get_max(vec).min(new_head);
  }


  pub fn move_to_start(&mut self) {
    self.head = 0;
  }


  pub fn move_to_end<T>(&mut self, vec: &Vec<T>) {
    self.head = self.get_max(vec);
  }


  pub fn move_wrapped<T>(&mut self, vec: &Vec<T>, mut delta: isize) {
    let imax = self.get_max(vec) as isize;
    let mut remainder = self.move_head(vec, delta);
    if remainder != 0 {
      self.move_head(vec, 
        vec.len() as isize * remainder.signum() * -1
      );
      self.move_wrapped(vec, 
        (remainder.saturating_abs() - 1) * remainder.signum()
      )
    }
  }


  pub fn move_head<T>(&mut self, vec: &Vec<T>, mut idelta: isize) -> isize {
    let ihead = self.head as isize;
    let imax = self.get_max(vec) as isize;
    let new_ihead = ihead + idelta;
    if new_ihead < 0 {
      self.head = 0;
      new_ihead
    } else if new_ihead > imax {
      self.head = self.get_max(vec);
      new_ihead - imax
    } else {
      self.head = new_ihead as usize;
      0
    }
  }


  pub fn remove<T>(&mut self, vec: &mut Vec<T>) -> bool {
    if self.head >= vec.len() {
      false
    } else {
      vec.remove(self.head);
      let head = self.head;
      self.move_wrapped(vec, -1);
      true
    }
  }


  pub fn delete<T>(&self, vec: &mut Vec<T>) -> bool {
    if self.head >= vec.len() {
      false
    } else {
      vec.remove(self.head);
      true
    } 
  }


  pub fn backspace<T>(&mut self, vec: &mut Vec<T>) -> bool {
    if self.peek_move(vec, -1) != 0 {
      false
    } else {
      self.move_head(vec, -1);
      vec.remove(self.head);
      true
    } 
  }


  pub fn insert_unique_with<T>(
    &mut self, vec: &mut Vec<T>, is_equal: impl Fn(&T) -> bool, unit: T
  ) -> bool {
    if let Some((idx, _)) = vec
      .iter_mut()
      .enumerate()
      .find(|(_, u)| is_equal(u))
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
    } else {
      self.head += 1;
      vec.insert(self.head, unit);
      true
    }
  }


  pub fn insert<T>(&mut self, vec: &mut Vec<T>, c: T) {
    if self.head + 1 == vec.len() || vec.len() == 0 {
      vec.push(c);
      self.move_head(vec, 1);
    } else {
      let head = self.head;
      vec.insert(self.head, c);
      self.move_head(vec, 1);
    }
  }


  pub fn get_weighted_head(&self, vec: &[char]) -> usize {
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
  pref_x: usize,
}


impl Point {
  pub fn editor<T>(mut self, vec: &Vec<Vec<T>>) -> Self {
    vec.get(self.y.head).map(|v| self.x = self.x.editor(v)); self
  }


  pub fn get_linear<T>(&self, vec: &Vec<Vec<T>>) -> usize {
    vec
      .iter()
      .take(self.y.head)
      .map(|v| v.len().max(1))
      .chain(std::iter::once(self.x.head))
      .sum()
  }


  pub fn set_linear<T>(&mut self, vec: &Vec<Vec<T>>, idx: usize) {
    self.y.move_to_start();
    self.x.move_to_start();
    self.move_x(vec, idx as isize);
  }


  pub fn move_y<T>(&mut self, vec: &Vec<Vec<T>>, delta: isize) -> bool {
    if self.y.move_head(vec, delta) == delta {
      false
    } else {
      vec.get(self.y.head).map(|v| self.x.fit(v, self.pref_x));
      true
    }
  }


  pub fn move_x<T>(&mut self, vec: &Vec<Vec<T>>, delta: isize) {
    let mut remainder = delta;

    while 0 != remainder
    && let Some(x_vec) = vec.get(self.y.head) 
    {
      remainder = self.x.move_head(x_vec, remainder);

      if 0 != remainder
      && 0 == self.y.move_head(vec, remainder.signum()) 
      && let Some(x_vec) = vec.get(self.y.head) 
      {
        self.x.move_head(
          x_vec, u32::MAX as isize * remainder.signum() * -1
        );
        remainder = remainder
          .saturating_abs()
          .saturating_sub(1) * remainder.signum();

      } else {
        remainder = 0;
      }
    }
    self.pref_x = self.x.head;
  }


  pub fn delete<T>(&mut self, vec: &mut Vec<Vec<T>>) -> bool {
    vec
      .get_mut(self.y.head)
      .map(|c| self.x.delete(c))
      .is_some()
  }


  pub fn backspace<T>(&mut self, vec: &mut Vec<Vec<T>>) -> bool {
    vec
      .get_mut(self.y.head)
      .map(|c| self.x.backspace(c))
      .is_some()
  }


  pub fn insert<T>(&mut self, vec: &mut Vec<Vec<T>>, t: T) -> bool {
    vec
      .get_mut(self.y.head)
      .map(|c| self.x.insert(c, t))
      .is_some()
  }


  pub fn move_left<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_x(vec, delta as isize * -1); true
  }
  pub fn move_right<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_x(vec, delta as isize); true
  }
  pub fn move_down<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_y(vec, delta as isize)
  }
  pub fn move_up<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_y(vec, delta as isize * -1)
  }
}


#[derive(Copy, Clone, Debug, Default)]
pub struct PointView {
  pos: Pos,
  x: CursorView,
  y: CursorView,
}


impl From<&PointView> for Rect {
  fn from(pv: &PointView) -> Self {
    Rect::from(pv.dim()).with_pos(pv.pos()) 
  }
}


impl From<&Rect> for PointView {
  fn from(rect: &Rect) -> Self {
    let rect = rect.clone();
    Self {
      x: CursorView::from_size(rect.w()),
      y: CursorView::from_size(rect.h()),
      pos: rect.pos(),
    }
  }
}


impl PointView {
  pub fn get_x_view(&self)   -> CursorView {self.x}
  pub fn get_y_view(&self)   -> CursorView {self.y}
  pub fn dim(&self)          -> Dim { Dim(self.x.size, self.y.size) }  
  pub fn pos(&self)          -> Pos { self.pos }
  pub fn get_x_cursor(&self) -> u16 { self.pos.x() + self.x.cursor }
  pub fn get_y_cursor(&self) -> u16 { self.pos.y() + self.y.cursor }
  pub fn get_width(&self)    -> u16   {self.x.size}
  pub fn get_height(&self)   -> u16   {self.y.size}
  pub fn get_x_scroll(&self) -> usize {self.x.scroll}
  pub fn get_y_scroll(&self) -> usize {self.y.scroll}


  pub fn resize(&mut self, matrix: &PointMatrix<char>, rect: &Rect) {
    let rect = rect.clone();
    self.pos = rect.pos();
    self.y.resize(matrix.point.y.head, rect.h());
    self.x.resize(matrix.get_weighted_x(), rect.w());
  }


  pub fn update(&mut self, matrix: &PointMatrix<char>) -> bool {
    let y = self.y.update(matrix.point.y.head);
    let x = self.x.update(matrix.get_weighted_x());
    x || y
  }


  pub fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor};
    w
      .queue(cursor::MoveTo(self.get_x_cursor(), self.get_y_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}


#[derive(Copy, Clone, Debug, Default)]
pub struct CursorView {
  pub head:   usize, // data head
  pub scroll: usize, // start of displayable data
  pub cursor: u16,   // on-screen cursor
  pub size:   u16,   // width or height of rectangle
}


impl CursorView {
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


  pub fn get_weighted_view(self, vec: &[char]) -> Vec<(u16, &char)> {
    use unicode_width::UnicodeWidthChar;
    let size = self.size;  
    let mut text = vec.iter().skip(self.scroll);
    let mut acc_size: u16 = 0;
    let mut result = vec![];
    while let Some(c) = text.next() && acc_size < size {
      let width = c.width().and_then(|w| u16::try_from(w).ok()).unwrap_or(0);
      acc_size += width;
      result.push((width, c));
    }
    result
  }


  // preserve cursor position if it still fits in the new bounds
  pub fn resize(&mut self, new_head: usize, new_size: u16) {
    // position of cursor on screen
    self.size = new_size;
    self.head = new_head;
    // go to beginning of line
    if new_head <= usize::from(new_size) {
      self.scroll = 0;
      self.cursor = u16::try_from(self.head).unwrap();
    // position must be lowered to fit within new bounds
    } else if self.cursor > new_size.saturating_sub(1) {
      self.cursor = self.size.saturating_sub(1);
      self.scroll = self.head - usize::from(self.size.saturating_sub(1));
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
      let max_delta = self.size
        .saturating_sub(1)
        .saturating_sub(self.cursor);
      // no scroll
      if delta_size < usize::from(max_delta) { 
        self.cursor += u16::try_from(delta_size).unwrap();
        self.head = new_head;
        false
      // scroll forward
      } else {
        self.scroll += delta_size - usize::from(max_delta);
        self.cursor += max_delta;
        self.head = new_head;
        true
      }
    // move backward
    } else { 
      let delta_size = self.head - new_head;
      // no scroll
      if delta_size <= usize::from(self.cursor) {
        self.cursor -= u16::try_from(delta_size).unwrap();
        self.head = new_head;
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


pub struct CursorVec<T> {
  pub cursor: Cursor,
  pub data: Vec<T>,
}
impl<T> Default for CursorVec<T> {
  fn default() -> Self { 
    Self { cursor: Cursor::default(), data: vec![] } 
  }
}
impl<T> From<Vec<T>> for CursorVec<T> {
  fn from(vec: Vec<T>) -> Self { 
    Self { cursor: Cursor::default(), data: vec } 
  }
}
impl<T> CursorVec<T> {
  pub fn move_head(&mut self, mut idelta: isize) -> isize {
    self.cursor.move_head(&self.data, idelta)
  }
  pub fn move_wrapped(&mut self, mut idelta: isize) {
    self.cursor.move_wrapped(&self.data, idelta)
  }
  pub fn get(&self) -> Option<&T> {
    self.data.get(self.cursor.head)
  }
  pub fn get_mut(&mut self) -> Option<&mut T> {
    self.data.get_mut(self.cursor.head)
  }
  pub fn remove(&mut self) -> bool {
    self.cursor.remove(&mut self.data)
  }
  pub fn delete(&mut self) -> bool {
    self.cursor.delete(&mut self.data)
  }
  pub fn backspace(&mut self) -> bool {
    self.cursor.backspace(&mut self.data)
  }
  pub fn insert(&mut self, t: T) {
    self.cursor.insert(&mut self.data, t)
  }
  pub fn insert_unique_with(
    &mut self, is_equal: impl Fn(&T) -> bool, unit: T
  ) -> bool {
    self.cursor.insert_unique_with(&mut self.data, is_equal, unit)
  }
}


pub struct PointMatrix<T> {
  pub point: Point,
  pub data: Vec<Vec<T>>,
}
impl<T> GetMaxHeight for PointMatrix<T> {
  fn get_max_height(&self) -> u16 { self.data.get_max_height() }
}
impl<T> Default for PointMatrix<T> {
  fn default() -> Self { 
    Self { point: Point::default(), data: vec![] } 
  }
}
impl<T> From<Vec<Vec<T>>> for PointMatrix<T> {
  fn from(matrix: Vec<Vec<T>>) -> Self {
    Self { point: Point::default(), data: matrix }
  }
}
impl PointMatrix<char> {
  pub fn get_weighted_x(&self) -> usize {
    if let Some(vec) = self.data.get(self.point.y.head) {
      self.point.x.get_weighted_head(vec)
    } else {0}
  }
}
impl<T> PointMatrix<T> {
  pub fn editor(mut self) -> Self {
    self.point = self.point.editor(&self.data); self
  }
  pub fn get_linear(&self) -> usize {
    self.point.get_linear(&self.data)
  }
  pub fn set_linear(&mut self, idx: usize) {
    self.point.set_linear(&self.data, idx)
  }
  pub fn move_y(&mut self, idelta: isize) -> bool {
    self.point.move_y(&self.data, idelta)
  }
  pub fn move_x(&mut self, idelta: isize) -> isize {
    self.point.move_x(&self.data, idelta); 0
  }
  pub fn delete(&mut self) -> bool {
    self.point.delete(&mut self.data)
  }
  pub fn backspace(&mut self) -> bool {
    self.point.backspace(&mut self.data)
  }
  pub fn insert(&mut self, t: T) -> bool {
    self.point.insert(&mut self.data, t)
  }
  pub fn move_left(&mut self, delta: usize) -> bool {
    self.point.move_left(&self.data, delta)
  }
  pub fn move_right(&mut self, delta: usize) -> bool {
    self.point.move_right(&self.data, delta)
  }
  pub fn move_down(&mut self, delta: usize) -> bool {
    self.point.move_down(&self.data, delta)
  }
  pub fn move_up(&mut self, delta: usize) -> bool {
    self.point.move_up(&self.data, delta)
  }
}
