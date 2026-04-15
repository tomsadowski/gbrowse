// src/text.rs

use crossterm::{
  QueueableCommand, 
  style::{
    SetAttribute, Attribute,
    SetForegroundColor, SetBackgroundColor, Color, 
  },
};
use std::{
  io::{self, Write}
};

pub fn wrap(text: Vec<char>, width: usize) -> Vec<Vec<char>> {
  let mut idx = usize::MIN;
  let mut vec: Vec<Vec<char>> = vec![];
  while idx < text.len() {
    let line: Vec<char> = {
      let text = &text[idx..];
      if text.len() <= width {
        text.to_vec()
      } else {
        // search for first whitespace from right
        let s: Vec<&char> = text[..width]
          .iter().rev().skip_while(|c| !c.is_whitespace()).collect();
        // no space found, return whole slice
        if s.len() == 0 {
          text[..width].iter().copied().collect()
        // space found, return up to that space
        } else {
          s.into_iter().rev().copied().collect()
        }
      }
    };
    idx += line.len();
    vec.push(line);
  }
  vec
}
pub trait Linear {
  fn len(&self) -> usize;
  fn head(&self) -> usize;
  fn head_mut(&mut self) -> &mut usize;

  fn max_head(&self) -> usize {
    self.len().saturating_sub(1)
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
  fn peek_backward(&self, step: usize) -> usize {
    if step > self.head() {
      step - self.head()
    } else {0}
  }
  fn peek_forward(&self, step: usize) -> usize {
    let max_head = self.max_head();
    if self.head() + step > max_head {
      self.head() + step - max_head
    } else {0}
  }
  fn backward(&mut self, mut step: usize) -> usize {
    if step > self.head() {
      step -= self.head();
      *self.head_mut() = 0;
      step
    } else {
      *self.head_mut() -= step;
      0
    }
  }
  fn forward(&mut self, mut step: usize) -> usize {
    if self.head() + step > self.max_head() {
      step = self.head() + step - self.max_head();
      *self.head_mut() = self.max_head();
      step
    } else {
      *self.head_mut() += step;
      0
    }
  }
  fn wrapping_backward(&mut self, step: usize) {
    if step > self.head() {
      self.end();
    } else {
      *self.head_mut() -= step;
    }
  }
  fn wrapping_forward(&mut self, step: usize) {
    if self.head() + step > self.max_head() {
      self.start();
    } else {
      *self.head_mut() += step;
    }
  }
}
pub trait Planar {
  fn x_len(&self) -> usize;
  fn x_head(&self) -> usize;
  fn y_len(&self) -> usize;
  fn y_head(&self) -> usize;
  fn y_head_mut(&mut self) -> &mut usize;
}
impl<P: Planar> Linear for P {
  fn len(&self) -> usize {
    self.y_len()
  }
  fn head(&self) -> usize {
    self.y_head()
  }
  fn head_mut(&mut self) -> &mut usize {
    self.y_head_mut()
  }
}
#[derive(Clone, Debug, Default)]
pub struct EditLine {
  pub head: usize,
  pub text: String,
}
impl From<&str> for EditLine {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.into()}
  }
}
impl Linear for EditLine {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn max_head(&self) -> usize {
    self.text.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
}
impl EditLine {
  pub fn delete(&mut self) -> bool {
    if self.head < self.text.len() {
      self.text.remove(self.head);
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.backward(1);
      self.text.remove(self.head);
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.head + 1 == self.text.len() || self.text.len() == 0 {
      self.text.push(c);
      self.forward(1);
      true
    } else {
      self.text.insert(self.head, c);
      self.forward(1);
      true
    }
  }
}
#[derive(Clone, Debug, Default)]
pub struct TextLine {
  pub head: usize,
  pub text: Vec<char>,
}
impl From<Vec<char>> for TextLine {
  fn from(item: Vec<char>) -> Self {
    Self {head: 0, text: item}
  }
}
impl Linear for TextLine {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
}
#[derive(Clone, Debug, Default)]
pub struct Style {
  pub underline: bool,
  pub bold: bool,
  pub fg:   Option<Color>,
  pub bg:   Option<Color>,
}
impl Style {
  pub fn write<W>(&self, writer: &mut W) -> io::Result<()> 
  where W: Write
  {
    if let Some(fg) = self.fg {
      writer.queue(SetForegroundColor(fg))?;
    }
    if let Some(bg) = self.bg {
      writer.queue(SetBackgroundColor(bg))?;
    }
    if self.bold {
      writer.queue(SetAttribute(Attribute::Bold))?;
    }
    if self.underline {
      writer.queue(SetAttribute(Attribute::Underlined))?;
    }
    Ok(())
  }
}
#[derive(Clone, Debug, Default)]
pub struct StyledText {
  pub style: Style,
  pub wrap:  bool,
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
impl StyledText {
  pub fn with_style(mut self, style: &Style) -> Self {
    self.style = style.clone();
    self
  }
  pub fn wrap(mut self, wrap: bool) -> Self {
    self.wrap = wrap;
    self
  }
  // get owned chars
  pub fn print(&self, width: usize) -> Vec<Vec<char>> {
    let text: Vec<char> = self.text.chars().collect();
    if text.len() == 0 {
      vec![vec![' ']]
    } else if self.wrap {
      wrap(text, width)
    } else {
      vec![text]
    }
  }
  // get indexed owned chars
  pub fn print_vec(vec: &Vec<Self>, width: usize) -> Vec<(usize, Vec<char>)> {
    vec.iter().enumerate().flat_map(
      |(idx, styled)| 
        styled.print(width)
          .into_iter()
          .map(move |text| (idx, text))
      ).collect()
  }
}
pub struct StyledTextPlane {
  pub head: usize,
  pub pref_x: usize,
  // usize: location of StyledText in source
  pub text: Vec<(usize, TextLine)>,
  pub source: Vec<StyledText>,
}
impl Default for StyledTextPlane {
  fn default() -> Self {
    Self {
      head: 0,
      pref_x: 0,
      text: vec![],
      source: vec![],
    }
  }
}
impl Planar for StyledTextPlane {
  fn x_len(&self) -> usize {
    self.text[self.head].1.len()
  }
  fn x_head(&self) -> usize {
    if self.text.len() > 0 {
      self.text[self.head].1.head()
    } else {0}
  }
  fn y_len(&self) -> usize {
    self.text.len()
  }
  fn y_head(&self) -> usize {
    self.head
  }
  fn y_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
}
impl StyledTextPlane {
  pub fn new(source: Vec<StyledText>, width: u16) -> Self {
    let text = 
      StyledText::print_vec(&source, usize::from(width))
        .into_iter()
        .map(|(idx, text)| (idx, TextLine::from(text)));
    Self {
      head: 0, 
      pref_x: 0, 
      text: text.collect(), 
      source
    }
  }
  pub fn get_source_idx(&self) -> usize {
    self.text[self.head()].0
  }
  pub fn get_idx(&self) -> usize {
    self.text[..self.head()]
      .iter()
      .map(|(_, line)| line.len().max(1))
      .chain(std::iter::once(self.x_head()))
      .sum()
  }
  fn set_idx(&mut self, idx: usize) {
    self.start();
    self.text[self.head].1.start();
    self.right(idx);
  }
  pub fn resize(&mut self, width: u16) {
    let idx = self.get_idx();
    self.text = 
      StyledText::print_vec(&self.source, usize::from(width))
        .into_iter()
        .map(|(idx, text)| (idx, TextLine::from(text)))
        .collect();
    self.set_idx(idx);
  }
  pub fn up(&mut self, step: usize) -> bool {
    let x = self.x_head();
    if self.backward(step) != step {
      self.text[self.head].1.fit(self.pref_x);
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    let x = self.x_head();
    if self.forward(step) != step {
      self.text[self.head].1.fit(self.pref_x);
      true
    } else {false}
  }
  pub fn left(&mut self, step: usize) -> usize {
    if self.text.len() == 0 {return step}
    let remainder = self.text[self.head].1.backward(step);
    if remainder == 0 {
      self.pref_x = self.text[self.head].1.head;
      0
    } else if self.backward(1) == 0 {
      self.text[self.head].1.end();
      self.left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
  pub fn right(&mut self, step: usize) -> usize {
    if self.text.len() == 0 {return step}
    let remainder = self.text[self.head].1.forward(step);
    if remainder == 0 {
      self.pref_x = self.text[self.head].1.head;
      0
    } else if self.forward(1) == 0 {
      self.text[self.head].1.start();
      self.right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}
