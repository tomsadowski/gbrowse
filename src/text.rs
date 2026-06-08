// src/text.rs

use crate::{
  view::ViewPort,
  cursor::{UnitCursor, UnitCursorMut, CursorPlane},
  style::{Style, TextStyle},
  util,
};
use unicode_width::UnicodeWidthChar;

pub trait RenderedText: From<(usize, Vec<char>)>
{
  fn get_origin_index(&self) -> usize;
}

#[derive(Clone, Debug, Default)]
pub struct TextLine {
  pub index:  usize,
  pub head:   usize,
  pub text:   Vec<char>,
}
impl From<&str> for TextLine {
  fn from(item: &str) -> Self {
    Self {
      index:  0,
      head: 0, 
      text: item.chars().collect()
    }
  }
}
impl From<(usize, Vec<char>)> for TextLine {
  fn from(item: (usize, Vec<char>)) -> Self {
    Self {
      head: 0, 
      index:  item.0,
      text: item.1
    }
  }
}
impl RenderedText for TextLine {
  fn get_origin_index(&self) -> usize {
    self.index
  }
}
impl UnitCursor for TextLine {
  type Unit = char;
  fn get_units(&self) -> &Vec<Self::Unit> {
    &self.text
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
    self.text.len().saturating_sub(1)
  }
}

#[derive(Clone, Debug, Default)]
pub struct EditLine {
  pub head: usize,
  pub index: usize,
  pub text: Vec<char>,
}
impl From<&str> for EditLine {
  fn from(item: &str) -> Self {
    let mut editline = Self {
      head:  0, 
      index: 0,
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
      index: 0,
      text:  item
    };
    editline.move_to_end();
    editline
  }
}
impl From<(usize, Vec<char>)> for EditLine {
  fn from(item: (usize, Vec<char>)) -> Self {
    let mut editline = Self {
      head:  0, 
      index: item.0,
      text:  item.1
    };
    editline.move_to_end();
    editline
  }
}
impl RenderedText for EditLine {
  fn get_origin_index(&self) -> usize {
    self.index
  }
}
impl ToString for EditLine {
  fn to_string(&self) -> String {
    self.text.iter().collect()
  }
}
impl UnitCursor for EditLine {
  type Unit = char;
  fn get_units(&self) -> &Vec<char> {
    &self.text
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
    self.text.len()
  }
}
impl UnitCursorMut for EditLine {
  fn units_mut(&mut self) -> &mut Vec<char> {
    &mut self.text
  }
}

#[derive(Clone, Debug, Default)]
pub struct StyledText {
  pub wrap:  bool,
  pub style: Style,
  pub text:  String,
}
impl From<&str> for StyledText {
  fn from(s: &str) -> Self {
    Self {
      wrap:  true,
      style: Style::default(),
      text:  s.into(),
    }
  }
}
impl From<String> for StyledText {
  fn from(s: String) -> Self {
    Self {
      wrap:  true,
      style: Style::default(),
      text:  s,
    }
  }
}
impl From<TextStyle> for StyledText {
  fn from(t: TextStyle) -> Self {
    Self {
      wrap:  t.wrap,
      style: t.style,
      text:  "".into(),
    }
  }
}
impl StyledText {
  pub fn text(mut self, text: &str) -> Self {
    self.text = text.into();
    self
  }

  pub fn style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    self
  }

  pub fn wrap(mut self, wrap: bool) -> Self {
    self.wrap = wrap;
    self
  }

  pub fn print(&self, width: usize) -> Vec<Vec<char>> {
    if self.text.len() == 0 {
      vec![vec![' ']]
    } else if self.wrap {
      self.text
        .lines()
        .flat_map(|line| util::get_wrapped_text(line, width))
        .collect()
    } else {
      self.text
        .lines()
        .map(|line| line.chars().collect())
        .collect()
    }
  }
}

pub fn get_rendered_text<T>(vec: &Vec<StyledText>, width: usize) -> Vec<T>
where T: RenderedText
{
  vec.iter().enumerate().flat_map(
    |(idx, styled)| 
      styled
        .print(width)
        .into_iter()
        .map(move |text| (idx, text).into())
    ).collect()
}

#[derive(Clone, Debug, Default)]
pub struct TextPlane<T> {
  pub text:   Vec<T>, 
  pub head:   usize,
  pub pref_x: usize,
}
impl<T> UnitCursor for TextPlane<T> {
  type Unit = T;
  fn get_units(&self) -> &Vec<Self::Unit> {
    &self.text
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
    self.text.len().saturating_sub(1)
  }
}
impl<T> UnitCursorMut for TextPlane<T> {
  fn units_mut(&mut self) -> &mut Vec<Self::Unit> {
    &mut self.text
  }
}
impl<T: RenderedText> From<(usize, Vec<char>)> for TextPlane<T> {
  fn from(item: (usize, Vec<char>)) -> Self {
    Self {
      head:   0, 
      pref_x: 0,
      text:   vec![item.into()],
    }
  }
}
impl<T: RenderedText> RenderedText for TextPlane<T> {
  fn get_origin_index(&self) -> usize {
    self
      .use_current(|c| c.get_origin_index())
      .unwrap_or(0)
  }
}
impl<T: RenderedText> TextPlane<T> {
  pub fn new<V: ViewPort>(view: &V, input: &Vec<StyledText>) -> Self {
    Self {
      text:   get_rendered_text(input, view.get_view_port().w.into()), 
      head:   0, 
      pref_x: 0, 
    }
  }
}
impl<T: UnitCursor> TextPlane<T> {
  pub fn move_up(&mut self, delta: usize) -> bool {
    if CursorPlane::move_up(self, delta) {
      let pref_x = self.pref_x;
      self.use_current_mut(|c| c.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if CursorPlane::move_down(self, delta) {
      let pref_x = self.pref_x;
      self.use_current_mut(|c| c.fit(pref_x));
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> usize {
    let remainder = CursorPlane::move_left(self, delta);
    if remainder == 0 {
      self.pref_x = self
        .use_current(|c| c.get_head())
        .unwrap_or(self.pref_x);
    } 
    remainder
  }

  pub fn move_right(&mut self, delta: usize) -> usize {
    let remainder = CursorPlane::move_right(self, delta);
    if remainder == 0 {
      self.pref_x = self
        .use_current(|c| c.get_head())
        .unwrap_or(self.pref_x);
    } 
    remainder
  }
}
