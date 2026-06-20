// src/cursor.rs

use crate::{
  GetRect,
  TextStyle,
  Rect,
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

impl<T> std::ops::Index<Cursor> for Vec<T> {
  type Output = T;
  fn index(&self, cursor: Cursor) -> &Self::Output {
    &self[*cursor]
  }
}

impl<T> std::ops::IndexMut<Cursor> for Vec<T> {
  fn index_mut(&mut self, cursor: Cursor) -> &mut Self::Output {
    &mut self[*cursor]
  }
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

  pub fn use_current<T, F, S>(&self, vec: &Vec<T>, func: F) -> Option<S>
  where F: Fn(&T) -> S
  {
    vec.get(self.head).map(|u| func(u))
  }

  pub fn if_current<T, F, S>(&self, vec: &Vec<T>, func: &mut F) -> Option<S>
  where F: FnMut(&T) -> S
  {
    vec.get(self.head).map(|u| func(u))
  }

  pub fn use_current_mut<T, F, S>(&self, vec: &mut Vec<T>, func: F) 
    -> Option<S>
  where F: Fn(&mut T) -> S
  {
    vec.get_mut(self.head).map(|u| func(u))
  }

//pub fn get_unit_view(&self, vec: Vec<T>, axis: LineCursorView) -> Vec<&T> {
//  self.data
//    .iter()
//    .skip(axis.get_scroll())
//    .take(axis.get_size().into())
//    .collect() 
//}

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

  pub fn move_forward_wrapped<T>(&mut self, vec: &Vec<T>, delta: usize) -> bool {
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
}

#[derive(Clone, Debug, Default)]
pub struct Gursor<T> {
  pub head: usize,
  pub data: Vec<T>,
  buff:     bool,
}

impl<T> std::ops::Deref for Gursor<T> {
  type Target = Vec<T>;
  fn deref(&self) -> &Self::Target {&self.data}
}

impl<T> std::ops::DerefMut for Gursor<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {&mut self.data}
}

impl ToString for Gursor<char> {
  fn to_string(&self) -> String {self.data.iter().collect()}
}

impl<T> From<Vec<T>> for Gursor<T> {
  fn from(item: Vec<T>) -> Self {
    Self {
      head:  0, 
      buff:  false,
      data:  item
    }
  }
}

impl<T> Gursor<T> {
  pub fn make_editor(&mut self) {
    self.buff = true;
    self.move_to_end();
  }

  pub fn get_max_head(&self) -> usize {
    match self.buff {
      true  => self.data.len(),
      false => self.data.len().saturating_sub(1),
    }
  }

  pub fn get_current(&self) -> Option<&T> {
    self.data.get(self.head)
  }

  pub fn get_current_mut(&mut self) -> Option<&mut T> {
    self.data.get_mut(self.head)
  }

  pub fn use_current<F, S>(&self, func: F) -> Option<S>
  where F: Fn(&T) -> S
  {
    self.get_current().map(|u| func(u))
  }

  pub fn use_current_mut<F, S>(&mut self, func: F) -> Option<S>
  where F: Fn(&mut T) -> S
  {
    self.get_current_mut().map(|u| func(u))
  }

  pub fn get_unit_view(&self, axis: LineCursorView) -> Vec<&T> {
    self.data
      .iter()
      .skip(axis.get_scroll())
      .take(axis.get_size().into())
      .collect() 
  }

  pub fn peek_backward(&self, delta: usize) -> usize {
    if delta > self.head {
      delta - self.head
    } else {0}
  }

  pub fn peek_forward(&self, delta: usize) -> usize {
    let max_head = self.get_max_head();
    if self.head + delta > max_head {
      self.head + delta - max_head
    } else {0}
  }

  pub fn fit(&mut self, new_head: usize) {
    self.head = self.get_max_head().min(new_head);
  }

  pub fn move_to_start(&mut self) {
    self.head = 0;
  }

  pub fn move_to_end(&mut self) {
    self.head = self.get_max_head();
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

  pub fn move_forward(&mut self, mut delta: usize) -> usize {
    if self.head + delta > self.get_max_head() {
      delta     = self.head + delta - self.get_max_head();
      self.head = self.get_max_head();
      delta
    } else {
      self.head += delta;
      0
    }
  }

  pub fn move_backward_wrapped(&mut self, delta: usize) -> bool {
    if self.data.len() <= 1 {
      false
    } else if delta > self.head {
      self.move_to_end();
      true
    } else {
      self.head -= delta;
      true
    }
  }

  pub fn move_forward_wrapped(&mut self, delta: usize) -> bool {
    if self.data.len() <= 1 {
      false
    } else if self.head + delta > self.get_max_head() {
      self.move_to_start();
      true
    } else {
      self.head += delta;
      true
    }
  }

  pub fn remove(&mut self) -> usize {
    if self.head < self.data.len() {
      self.data.remove(self.head);
      self.move_backward_wrapped(1);
    }
    self.data.len()
  }

  pub fn insert_or_move<F>(&mut self, func: F, unit: T) -> bool
  where F: Fn(&T) -> bool,
  {
    if let Some((idx, _)) = self.data
      .iter_mut()
      .enumerate()
      .find(|(_, u)| func(u))
    {
      self.head = idx;
      false
    } else if self.data.len() == 0 {
      self.data.push(unit);
      true
    } else if self.head + 1 == self.data.len() {
      self.data.push(unit);
      self.head += 1;
      true
    }
    else {
      self.head += 1;
      self.data.insert(self.head, unit);
      true
    }
  }

  pub fn delete(&mut self) -> bool {
    if self.head < self.data.len() {
      self.data.remove(self.head);
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.move_backward(1);
      self.data.remove(self.head);
      true
    } else {false}
  }

  pub fn insert(&mut self, c: T) -> bool {
    if self.head + 1 == self.data.len() || self.data.len() == 0 {
      self.data.push(c);
      self.move_forward(1);
      true
    } else {
      self.data.insert(self.head, c);
      self.move_forward(1);
      true
    }
  }
}

impl Gursor<char> {
  pub fn get_weighted_head(&self) -> usize {
    use unicode_width::UnicodeWidthChar;
    self
      .iter()
      .take(self.head)
      .map(|c| c.width().unwrap_or(0))
      .sum()
  }

  pub fn get_weighted_length(&self) -> usize {
    use unicode_width::UnicodeWidthChar;
    self
      .iter()
      .map(|c| c.width().unwrap_or(0))
      .sum()
  }

  pub fn get_weighted_view(&self, axis: LineCursorView) -> Vec<&char> {
    use unicode_width::UnicodeWidthChar;
    let size         = usize::from(axis.get_size());  
    let mut text     = self.iter().skip(axis.get_scroll());
    let mut acc_size = 0;
    let mut result   = vec![];
    while let Some(c) = text.next() && acc_size < size {
      acc_size += c.width().unwrap_or(0);
      result.push(c);
    }
    result
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CursorPlane {
  pub x: Cursor,
  pub y: Cursor,
}

impl CursorPlane {
  pub fn editor(mut self) -> Self {
    self.make_editor();
    self
  }

  pub fn make_editor(&mut self) {
    self.x.make_editor();
  }

  pub fn get_linear_head(&self) -> usize {
    match self.use_current(|c| c.head) {
      None         => 0,
      Some(x_head) => self.data[..self.head]
        .iter()
        .map(|line| line.data.len().max(1))
        .chain(std::iter::once(x_head))
        .sum(),
    }
  }

  pub fn set_linear_head(&mut self, idx: usize) {
    self.y.move_to_start();
    self.use_current_mut(|c| c.move_to_start());
    self.move_right(idx);
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.move_backward(delta) != delta {
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.move_forward(delta) != delta {
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_left<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> usize {
    let Some(remainder) = 
      vec.get(*self.y).map(|_| self.x.move_backward(delta)) 
      else {return delta};
    if remainder != 0 && self.y.move_backward(1) == 0 {
      let Some(_) = 
        vec.get(*self.y).map(|v| self.x.move_to_end(v)) 
        else {return remainder};
      self.move_left(vec, remainder.saturating_sub(1))
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

impl<T> Gursor<Gursor<T>> {
  pub fn make_editor_lines(&mut self) {
    for cursor in self.data.iter_mut() {
      cursor.make_editor();
    }
  }

  pub fn get_linear_head(&self) -> usize {
    match self.use_current(|c| c.head) {
      None         => 0,
      Some(x_head) => self.data[..self.head]
        .iter()
        .map(|line| line.data.len().max(1))
        .chain(std::iter::once(x_head))
        .sum(),
    }
  }

  pub fn set_linear_head(&mut self, idx: usize) {
    self.move_to_start();
    self.use_current_mut(|c| c.move_to_start());
    self.move_right(idx);
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.move_backward(delta) != delta {
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    let pref_x = self.use_current_mut(|current| current.head).unwrap_or(0);
    if self.move_forward(delta) != delta {
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> usize {
    let remainder = self
      .use_current_mut(|c| c.move_backward(delta))
      .unwrap_or(delta);
    if remainder != 0 && self.move_backward(1) == 0 {
      self.use_current_mut(|c| c.move_to_end());
      self.move_left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }

  pub fn move_right(&mut self, delta: usize) -> usize {
    let remainder = self
      .use_current_mut(|c| c.move_forward(delta))
      .unwrap_or(delta);
    if remainder != 0 && self.move_forward(1) == 0 {
      self.use_current_mut(|c| c.move_to_start());
      self.move_right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct IndexedCursor<T> {
  pub matrix:   Gursor<T>, 
  pub indexes:  Vec<usize>,
}

impl<T> std::ops::Deref for IndexedCursor<T> {
  type Target = Gursor<T>;
  fn deref(&self) -> &Self::Target {&self.matrix}
}

impl<T> std::ops::DerefMut for IndexedCursor<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {&mut self.matrix}
}

impl<T> IndexedCursor<T> {
  pub fn get_current_index(&self) -> usize {
    self.indexes.get(self.matrix.head)
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }

  pub fn get_view(&self, axis: LineCursorView) -> Vec<(&usize, &T)> {
    self.indexes
      .iter()
      .zip(self.matrix.iter())
      .skip(axis.get_scroll())
      .take(usize::from(axis.get_size()))
      .collect()
  }
}

impl IndexedCursor<Gursor<char>> {
  pub fn print(
    width:  usize, 
    text:   &Vec<String>,
    styles: &Vec<TextStyle>
  ) -> Self {
    let (indexes, cursors): (Vec<usize>, Vec<Gursor<char>>) = styles
      .iter()
      .zip(text.iter())
      .enumerate()
      .flat_map(
      |(idx, (style, text))| 
        style
          .print(width, text)
          .into_iter()
          .map(move |text| (idx, text.into()))
      ).unzip();
    Self {
      matrix: cursors.into(), 
      indexes,
    }
  }

  pub fn print_from<V: GetRect>(
    view:   &V, 
    text:   &Vec<String>,
    styles: &Vec<TextStyle>
  ) -> Self {
    let width = usize::from(view.get_rect().w);
    Self::print(width, text, styles)
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenCursor {
  x: LineCursorView,
  y: LineCursorView,
}

impl<V: GetRect> From<&V> for ScreenCursor {
  fn from(view: &V) -> Self {
    let view = view.get_rect();
    Self {
      x: LineCursorView::new(view.x, view.w),
      y: LineCursorView::new(view.y, view.h),
    }
  }
}

impl GetRect for ScreenCursor {
  fn get_rect(&self) -> Rect {
    Rect {
      x: self.x.get_start(),
      y: self.y.get_start(),
      w: self.x.get_size(),
      h: self.y.get_size(),
    }
  }
}

impl ScreenCursor {
  pub fn get_x_view(&self)   -> LineCursorView {self.x}
  pub fn get_y_view(&self)   -> LineCursorView {self.y}
  pub fn get_x_cursor(&self) -> u16            {self.x.get_cursor()}
  pub fn get_y_cursor(&self) -> u16            {self.y.get_cursor()}
  pub fn get_width(&self)    -> u16            {self.x.get_size()}
  pub fn get_height(&self)   -> u16            {self.y.get_size()}
  pub fn get_x_scroll(&self) -> usize          {self.x.get_scroll()}
  pub fn get_y_scroll(&self) -> usize          {self.y.get_scroll()}

  pub fn resize<V>(&mut self, matrix: &Gursor<Gursor<char>>, view: &V)
  where V: GetRect,
  {
    let rect = view.get_rect();
    self.y.resize(matrix.head, rect.y, rect.h);
    self.x.resize(
      matrix.use_current(|c| c.get_weighted_head()).unwrap_or(0), 
      rect.x, 
      rect.w
    );
  }

  pub fn update(&mut self, matrix: &Gursor<Gursor<char>>) -> bool {
    let y = self.y.update(matrix.head);
    let x = self.x.update(
      matrix.use_current(|c| c.get_weighted_head()).unwrap_or(0)
    );
    x || y
  }
}

impl crate::Draw for ScreenCursor {
  fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor};
    w
      .queue(cursor::MoveTo(self.x.get_cursor(), self.y.get_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LineCursorView {
  head:   usize, // data head
  scroll: usize, // start of displayable data
  cursor: u16,   // on-screen cursor
  start:  u16,   // x or y
  size:   u16,   // width or height of rectangle
}

impl LineCursorView {
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
  pub fn get_cursor(&self) -> u16   {self.cursor}
  pub fn get_size(&self)   -> u16   {self.size}
  pub fn get_start(&self)  -> u16   {self.start}

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
