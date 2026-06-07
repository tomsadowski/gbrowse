// src/text.rs

use crate::{
  view::ViewPort,
  cursor::{UnitCursor, UnitCursorMut},
  style::{Style, TextStyle},
  util,
};
use unicode_width::UnicodeWidthChar;


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
impl From<Vec<char>> for TextLine {
  fn from(item: Vec<char>) -> Self {
    Self {
      index:  0,
      head: 0, 
      text: item
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
  pub text: Vec<char>,
}
impl From<&str> for EditLine {
  fn from(item: &str) -> Self {
    let mut editline = Self {
      head: 0, 
      text: item.chars().collect()
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
  pub fn with_text(mut self, text: &str) -> Self {
    self.text = text.into();
    self
  }
  pub fn with_style<T>(mut self, style: T) -> Self 
  where T: Into<Style> + Copy
  {
    self.style = style.into();
    self
  }
  pub fn with_wrap(mut self, wrap: bool) -> Self {
    self.wrap = wrap;
    self
  }

  // get owned chars
  pub fn print(&self, width: usize) -> Vec<Vec<char>> {
    if self.wrap {
      util::get_wrapped_text(&self.text, width)
    } else {
      vec![self.text.chars().collect()]
    }
  }

  pub fn get_textlines(vec: &Vec<Self>, width: usize) -> Vec<TextLine> {
    vec.iter().enumerate().flat_map(
      |(idx, styled)| 
        styled
          .print(width)
          .into_iter()
          .map(move |text| (idx, text).into())
      ).collect()
  }
}

#[derive(Clone, Debug, Default)]
pub struct StyledTextPlane {
  pub source: Vec<StyledText>,
  pub text:   Vec<TextLine>, 
  pub head:   usize,
  pub pref_x: usize,
}
impl UnitCursor for StyledTextPlane {
  type Unit = TextLine;
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
impl UnitCursorMut for StyledTextPlane {
  fn units_mut(&mut self) -> &mut Vec<Self::Unit> {
    &mut self.text
  }
}
impl StyledTextPlane {
  pub fn new<V, I, F>(view: &V, input: &Vec<I>, func: F) -> Self 
  where 
    V: ViewPort,
    F: Fn(&I) -> StyledText,
  {
    let source = input.iter().map(|i| func(i)).collect();
    Self {
      head:   0, 
      pref_x: 0, 
      text: StyledText::get_textlines(&source, view.get_view_port().w.into()), 
      source,
    }
  }

  pub fn get_source_index(&self) -> usize {
    self
      .get_current()
      .map(|t| t.index)
      .unwrap_or(0)
  }

  pub fn get_source(&self) -> String {
    self.source
      .get(self.get_source_index())
      .map(|t| t.text.clone())
      .unwrap_or("empty".into())
  }

  fn get_index(&self) -> usize {
    match self.get_current().map(|u| u.get_head()) {
      None         => 0,
      Some(x_head) => self.text[..self.get_head()]
        .iter()
        .map(|line| line.get_length().max(1))
        .chain(std::iter::once(x_head))
        .sum(),
    }
  }

  fn set_index(&mut self, idx: usize) {
    self.move_to_start();
    self.get_current_mut().map(|t| t.move_to_start());
    self.move_right(idx);
  }

  pub fn restyle<V, I, F>(&mut self, view: V, input: &Vec<I>, func: F) 
  where 
    V: ViewPort,
    F: Fn(&I) -> StyledText,
  {
    self.source = input.iter().map(|i| func(i)).collect();
    let idx   = self.get_source_index();
    self.text = StyledText::get_textlines(
      &self.source, 
      view.get_view_port().w.into()
    );
    self.set_index(idx);
  }

  pub fn resize(&mut self, width: u16) {
    let idx   = self.get_index();
    self.text = StyledText::get_textlines(&self.source, width.into());
    self.set_index(idx);
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    if self.move_backward(delta) != delta {
      self.text[self.head].fit(self.pref_x);
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if self.move_forward(delta) != delta {
      self.text[self.head].fit(self.pref_x);
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> usize {
    if self.text.len() == 0 {return delta}
    let remainder = self.text[self.head].move_backward(delta);
    if remainder == 0 {
      self.pref_x = self.text[self.head].get_head();
      0
    } else if self.move_backward(1) == 0 {
      self.text[self.head].move_to_end();
      self.move_left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }

  pub fn move_right(&mut self, delta: usize) -> usize {
    if self.text.len() == 0 {return delta}
    let remainder = self.text[self.head].move_forward(delta);
    if remainder == 0 {
      self.pref_x = self.text[self.head].get_head();
      0
    } else if self.move_forward(1) == 0 {
      self.text[self.head].move_to_start();
      self.move_right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}
