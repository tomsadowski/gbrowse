// src/cursor.rs

use crate::view::Rect;
use unicode_width::UnicodeWidthChar;
use std::slice;
use std::iter::Take;
use std::io::Write;


pub trait UnitCursor {
  type Unit;
  fn units(&self)        -> &Vec<Self::Unit>;
  fn head(&self)         -> usize;
  fn head_mut(&mut self) -> &mut usize;
  fn max_head(&self)     -> usize;

  fn length(&self) -> usize {
    self.units().len()
  }

  fn current(&self) -> &Self::Unit {
    &self.units()[self.head()]
  }

  fn current_checked(&self) -> Option<&Self::Unit> {
    self.units().get(self.head())
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

  fn view_units(&self, start: usize, width: usize) 
    -> Take<slice::Iter<'_, Self::Unit>> 
  {
    if start >= self.length() {
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

  fn current_mut(&mut self) -> &mut Self::Unit {
    let head = self.head();
    &mut self.units_mut()[head]
  }

  fn current_mut_checked(&mut self) -> Option<&mut Self::Unit> {
    let head = self.head();
    self.units_mut().get_mut(head)
  }

  fn remove(&mut self) -> usize {
    let head = self.head();
    if head < self.length() {
      self.units_mut().remove(head);
      if self.units().len() > 0 {
        self.wrapping_backward(1);
      }
    }
    self.length()
  }

  // maybe return bool
  fn insert_or_move<F>(&mut self, func: F, unit: Self::Unit) 
  where F: Fn(&Self::Unit) -> bool,
  {
    // search for tab with same url_str
    // move head to location of tab with url_str
    if let Some((idx, _)) = self.units_mut()
      .iter_mut()
      .enumerate()
      .find(|(_, u)| func(u))
    {
      *self.head_mut() = idx;
    } else {
      if self.length() == 0 {
        self.units_mut().push(unit);
      } else if self.head() + 1 == self.length() {
        self.units_mut().push(unit);
        *self.head_mut() += 1;
      }
      else {
        *self.head_mut() += 1;
        let head = self.head();
        self.units_mut().insert(head, unit);
      }
    }
  }

  fn delete(&mut self) -> bool {
    let head = self.head();
    if head < self.length() {
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
    if head + 1 == self.length() || self.length() == 0 {
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
  fn view_weighted(&self, start: usize, width: usize) 
    -> Take<slice::Iter<'_, Self::Unit>>;
}
impl<U> WeightedCursor for U 
where 
  U: UnitCursor<Unit = char> 
{
  fn weighted_head(&self) -> usize {
    self
      .units()[..self.head()]
      .iter()
      .fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }

  fn weighted_len(&self) -> usize {
    self
      .units()
      .iter()
      .fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }

  fn view_weighted(&self, start: usize, width: usize) 
    -> Take<slice::Iter<'_, Self::Unit>> 
  {
    if start >= self.length() {
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
