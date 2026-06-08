// src/cursor.rs

use unicode_width::UnicodeWidthChar;


pub trait UnitCursor {
  type Unit;

  fn get_units(&self) -> &Vec<Self::Unit>;

  fn get_head(&self) -> usize;

  fn get_head_mut(&mut self) -> &mut usize;

  fn get_max_head(&self) -> usize;

  fn get_length(&self) -> usize {
    self.get_units().len()
  }

  fn get_current(&self) -> Option<&Self::Unit> {
    self.get_units().get(self.get_head())
  }

  fn use_current<F, T>(&self, func: F) -> Option<T>
  where F: Fn(&Self::Unit) -> T
  {
    self.get_current().map(|u| func(u))
  }

  fn get_unit_view(&self, start: usize, width: usize) -> Vec<&Self::Unit> {
    self.get_units().iter().skip(start).take(width).collect() 
  }

  fn peek_backward(&self, delta: usize) -> usize {
    if delta > self.get_head() {
      delta - self.get_head()
    } else {0}
  }

  fn peek_forward(&self, delta: usize) -> usize {
    let max_head = self.get_max_head();
    if self.get_head() + delta > max_head {
      self.get_head() + delta - max_head
    } else {0}
  }

  fn fit(&mut self, new_head: usize) {
    *self.get_head_mut() = self.get_max_head().min(new_head);
  }

  fn move_to_start(&mut self) {
    *self.get_head_mut() = 0;
  }

  fn move_to_end(&mut self) {
    *self.get_head_mut() = self.get_max_head();
  }

  fn move_backward(&mut self, mut delta: usize) -> usize {
    if delta > self.get_head() {
      delta -= self.get_head();
      *self.get_head_mut() = 0;
      delta
    } else {
      *self.get_head_mut() -= delta;
      0
    }
  }

  fn move_forward(&mut self, mut delta: usize) -> usize {
    if self.get_head() + delta > self.get_max_head() {
      delta = self.get_head() + delta - self.get_max_head();
      *self.get_head_mut() = self.get_max_head();
      delta
    } else {
      *self.get_head_mut() += delta;
      0
    }
  }

  fn move_backward_wrapped(&mut self, delta: usize) -> bool {
    if self.get_length() <= 1 {
      false
    } else if delta > self.get_head() {
      self.move_to_end();
      true
    } else {
      *self.get_head_mut() -= delta;
      true
    }
  }

  fn move_forward_wrapped(&mut self, delta: usize) -> bool {
    if self.get_length() <= 1 {
      false
    } else if self.get_head() + delta > self.get_max_head() {
      self.move_to_start();
      true
    } else {
      *self.get_head_mut() += delta;
      true
    }
  }
}

pub trait UnitCursorMut: UnitCursor {
  fn units_mut(&mut self) -> &mut Vec<Self::Unit>;

  fn get_current_mut(&mut self) -> Option<&mut Self::Unit> {
    let head = self.get_head();
    self.units_mut().get_mut(head)
  }

  fn use_current_mut<F, T>(&mut self, func: F) -> Option<T>
  where F: Fn(&mut Self::Unit) -> T
  {
    self.get_current_mut().map(|u| func(u))
  }

  fn remove(&mut self) -> usize {
    let head = self.get_head();
    if head < self.get_length() {
      self.units_mut().remove(head);
      self.move_backward_wrapped(1);
    }
    self.get_length()
  }

  fn insert_or_move<F>(&mut self, func: F, unit: Self::Unit) -> bool
  where F: Fn(&Self::Unit) -> bool,
  {
    if let Some((idx, _)) = self.units_mut()
      .iter_mut()
      .enumerate()
      .find(|(_, u)| func(u))
    {
      *self.get_head_mut() = idx;
      false
    } else if self.get_length() == 0 {
      self.units_mut().push(unit);
      true
    } else if self.get_head() + 1 == self.get_length() {
      self.units_mut().push(unit);
      *self.get_head_mut() += 1;
      true
    }
    else {
      *self.get_head_mut() += 1;
      let head = self.get_head();
      self.units_mut().insert(head, unit);
      true
    }
  }

  fn delete(&mut self) -> bool {
    let head = self.get_head();
    if head < self.get_length() {
      self.units_mut().remove(head);
      true
    } else {false}
  }

  fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.move_backward(1);
      let head = self.get_head();
      self.units_mut().remove(head);
      true
    } else {false}
  }

  fn insert(&mut self, c: Self::Unit) -> bool {
    let head = self.get_head();
    if head + 1 == self.get_length() || self.get_length() == 0 {
      self.units_mut().push(c);
      self.move_forward(1);
      true
    } else {
      self.units_mut().insert(head, c);
      self.move_forward(1);
      true
    }
  }
}

pub trait WeightedCursor: UnitCursor {
  fn get_weighted_head(&self) -> usize;

  fn get_weighted_length(&self) -> usize;

  fn get_weighted_view(&self, start: usize, width: usize) 
    -> Vec<&Self::Unit>;
}
impl<U> WeightedCursor for U 
where U: UnitCursor<Unit = char> 
{
  fn get_weighted_head(&self) -> usize {
    self
      .get_units()
      .iter()
      .take(self.get_head())
      .map(|u| u.width().unwrap_or(0))
      .sum()
  }

  fn get_weighted_length(&self) -> usize {
    self
      .get_units()
      .iter()
      .map(|u| u.width().unwrap_or(0))
      .sum()
  }

  fn get_weighted_view(&self, start: usize, width: usize) 
    -> Vec<&Self::Unit> 
  {
    let mut text      = self.get_units().iter().skip(start);
    let mut acc_width = 0;
    let mut result    = vec![];
    while let Some(c) = text.next() && acc_width < width {
      acc_width += &c.width().unwrap_or(0);
      result.push(c);
    }
    result
  }
}

pub trait CursorPlane {
  fn get_index(&self) -> usize;

  fn set_index(&mut self, idx: usize);

  fn move_up(&mut self, delta: usize) -> bool;

  fn move_down(&mut self, delta: usize) -> bool;

  fn move_left(&mut self, delta: usize) -> usize;

  fn move_right(&mut self, delta: usize) -> usize;
}
impl<U, T> CursorPlane for U 
where 
  U: UnitCursorMut<Unit = T>,
  T: UnitCursor,
{
  fn get_index(&self) -> usize {
    match self.use_current(|c| c.get_head()) {
      None         => 0,
      Some(x_head) => self.get_units()[..self.get_head()]
        .iter()
        .map(|line| line.get_length().max(1))
        .chain(std::iter::once(x_head))
        .sum(),
    }
  }

  fn set_index(&mut self, idx: usize) {
    self.move_to_start();
    self.use_current_mut(|c| c.move_to_start());
    self.move_right(idx);
  }

  fn move_up(&mut self, delta: usize) -> bool {
    let x_head = self
      .use_current(|c| c.get_head())
      .unwrap_or(0);
    if self.move_backward(delta) != delta {
      self.use_current_mut(|c| c.fit(x_head));
      true
    } else {false}
  }

  fn move_down(&mut self, delta: usize) -> bool {
    let x_head = self
      .use_current(|c| c.get_head())
      .unwrap_or(0);
    if self.move_forward(delta) != delta {
      self.use_current_mut(|c| c.fit(x_head));
      true
    } else {false}
  }

  fn move_left(&mut self, delta: usize) -> usize {
    let remainder = self
      .use_current_mut(|c| c.move_backward(delta))
      .unwrap_or(delta);
    if remainder != 0 && self.move_backward(1) == 0 {
      self.use_current_mut(|c| c.move_to_end());
      self.move_left(remainder.saturating_sub(1))
    } else {remainder}
  }

  fn move_right(&mut self, delta: usize) -> usize {
    let remainder = self
      .use_current_mut(|c| c.move_forward(delta))
      .unwrap_or(delta);
    if remainder != 0 && self.move_forward(1) == 0 {
      self.use_current_mut(|c| c.move_to_start());
      self.move_right(remainder.saturating_sub(1))
    } else {remainder}
  }
}
