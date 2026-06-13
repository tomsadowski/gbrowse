// src/textcursor.rs

use crate::{
  UnitCursor, 
  UnitCursorMut, 
  CursorPlane,
  StyledText,
  ViewPort,
  LineCursor,
};


#[derive(Clone, Debug, Default)]
pub struct TextLine {
  pub head:   usize,
  pub text:   Vec<char>,
}

impl From<&str> for TextLine {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.chars().collect()}
  }
}

impl From<Vec<char>> for TextLine {
  fn from(item: Vec<char>) -> Self {
    Self {head: 0, text: item}
  }
}

impl UnitCursor for TextLine {
  type Unit = char;
  fn get_units(&self)        -> &Vec<Self::Unit> {&self.text}
  fn get_head_mut(&mut self) -> &mut usize       {&mut self.head}
  fn get_head(&self)         -> usize            {self.head}
  fn get_max_head(&self)     -> usize {
    self.text.len().saturating_sub(1)
  }
}


#[derive(Clone, Debug, Default)]
pub struct EditLine {
  pub head: usize,
  pub text: Vec<char>,
}

impl From<&str> for EditLine {
  fn from(item: &str) -> Self {
    let mut editline = Self {
      head:  0, 
      text:  item.chars().collect()
    };
    editline.move_to_end();
    editline
  }
}

impl From<Vec<char>> for EditLine {
  fn from(item: Vec<char>) -> Self {
    let mut editline = Self {
      head:  0, 
      text:  item
    };
    editline.move_to_end();
    editline
  }
}

impl ToString for EditLine {
  fn to_string(&self) -> String {
    self.text.iter().collect()
  }
}

impl UnitCursor for EditLine {
  type Unit = char;
  fn get_units(&self)        -> &Vec<char> {&self.text}
  fn get_head_mut(&mut self) -> &mut usize {&mut self.head}
  fn get_head(&self)         -> usize      {self.head}
  fn get_max_head(&self)     -> usize      {self.text.len()}
}

impl UnitCursorMut for EditLine {
  fn units_mut(&mut self)    -> &mut Vec<char> {&mut self.text}
}


#[derive(Clone, Debug, Default)]
pub struct TextPlane<T> {
  pub text:     Vec<T>, 
  pub indexes:  Vec<usize>,
  pub head:     usize,
  pub pref_x:   usize,
}

impl<T> UnitCursor for TextPlane<T> {
  type Unit = T;
  fn get_units(&self) -> &Vec<Self::Unit> {&self.text}
  fn get_head_mut(&mut self) -> &mut usize {&mut self.head}
  fn get_head(&self) -> usize {self.head}
  fn get_max_head(&self) -> usize {
    self.text.len().saturating_sub(1)
  }
}

impl<T> UnitCursorMut for TextPlane<T> {
  fn units_mut(&mut self) -> &mut Vec<Self::Unit> {&mut self.text}
}

impl<T> TextPlane<T> {
  pub fn get_current_reference_index(&self) -> usize {
    self.indexes.get(self.get_head())
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }

  pub fn get_view(&self, axis: LineCursor) -> Vec<(&usize, &T)> {
    let start   = axis.get_scroll();
    let size    = usize::from(axis.get_size());
    let indexes = self.indexes.iter().skip(start).take(size); 
    let units   = self.get_units().iter().skip(start).take(size);
    indexes.zip(units).collect()
  }
}

impl<T: From<Vec<char>>> TextPlane<T> {
  pub fn new<V: ViewPort>(view: &V, input: &Vec<StyledText>) -> Self {
    let width    = usize::from(view.get_view_port().w);
    let rendered = input.iter().enumerate().flat_map(
      |(idx, styled)| 
        styled
          .print(width)
          .into_iter()
          .map(move |text| (idx, text.into()))
      );
    let (indexes, text) = rendered.unzip();
    Self {
      head:   0, 
      pref_x: 0, 
      indexes,
      text, 
    }
  }
}

impl<T: UnitCursor> TextPlane<T> {
  pub fn move_up(&mut self, delta: usize) -> bool {
    if CursorPlane::move_up(self, delta) {
      let pref_x = self.pref_x;
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if CursorPlane::move_down(self, delta) {
      let pref_x = self.pref_x;
      self.use_current_mut(|current| current.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> usize {
    let remainder = CursorPlane::move_left(self, delta);
    if remainder == 0 {
      self.pref_x = self
        .use_current(|current| current.get_head())
        .unwrap_or(self.pref_x);
    } 
    remainder
  }

  pub fn move_right(&mut self, delta: usize) -> usize {
    let remainder = CursorPlane::move_right(self, delta);
    if remainder == 0 {
      self.pref_x = self
        .use_current(|current| current.get_head())
        .unwrap_or(self.pref_x);
    } 
    remainder
  }
}
