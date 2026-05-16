// src/cursor.rs

use crate::common as c;
use crossterm::{
  QueueableCommand, 
  style::{SetAttribute, Attribute},
  cursor::{self, MoveTo},
};
use unicode_width::UnicodeWidthChar;
use std::io::{self, Write};


pub trait UnitCursor {
  type Unit;
  fn units(&self)           -> &Vec<Self::Unit>;
  fn head(&self)            -> usize;
  fn head_mut(&mut self)    -> &mut usize;
  fn max_head(&self)        -> usize;
  
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
  fn iter_from(&self, shift: usize) -> std::slice::Iter<'_, Self::Unit> {
    let shift = std::cmp::min(shift, self.units().len().saturating_sub(1));
    self.units()[shift..].iter()
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
  fn weighted_range(&self, a: usize, b: usize) -> usize;
}
impl<U> WeightedCursor for U where U: UnitCursor<Unit = char> {
  fn weighted_head(&self) -> usize {
    self.units()[..self.head()].iter()
      .fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }
  fn weighted_len(&self) -> usize {
    self.units().iter()
      .fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }
  fn weighted_range(&self, a: usize, b: usize) -> usize {
    self.units()[a..b].iter()
      .fold(0, |acc, u| acc + u.width().unwrap_or(0))
  }
}
