// src/text.rs

use crate::{
  cursor::{UnitCursor, UnitCursorMut},
};
use crossterm::{
  Command, QueueableCommand, 
  style::{
    SetStyle, ContentStyle, SetAttribute, Attribute, Attributes,
    SetForegroundColor, SetBackgroundColor, Color, 
  },
};
use unicode_width::UnicodeWidthChar;
use std::{
  fmt,
  io::{self, Write},
  ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, Default)]
pub struct EditLine {
  pub head: usize,
  pub text: Vec<char>,
}
impl From<&str> for EditLine {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.chars().collect()}
  }
}
impl ToString for EditLine {
  fn to_string(&self) -> String {
    self.text.iter().collect()
  }
}
impl UnitCursor for EditLine {
  type Unit = char;
  fn units(&self) -> &Vec<char> {
    &self.text
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
  fn max_head(&self) -> usize {
    self.text.len()
  }
}
impl UnitCursorMut for EditLine {
  fn units_mut(&mut self) -> &mut Vec<char> {
    &mut self.text
  }
}
#[derive(Clone, Debug, Default)]
pub struct TextLine {
  pub head: usize,
  pub text: Vec<char>,
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
  fn units(&self) -> &Vec<char> {
    &self.text
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
  fn max_head(&self) -> usize {
    self.text.len().saturating_sub(1)
  }
}
#[derive(Clone, Debug, Default)]
pub struct Style {
  pub underline: bool,
  pub bold:      bool,
  pub fg:        Option<Color>,
  pub bg:        Option<Color>,
}
impl Command for Style {
  fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
    let mut contentstyle = ContentStyle::new();
    contentstyle.foreground_color = self.fg;
    contentstyle.background_color = self.bg;
    let mut attributes = Attributes::none();
    if self.bold {
      attributes.set(Attribute::Bold);
    }
    if self.underline {
      attributes.set(Attribute::Underlined);
    }
    contentstyle.attributes = attributes;
    SetStyle(contentstyle).write_ansi(f)?;
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
  pub fn wrap_text(text: Vec<char>, width: usize) -> Vec<Vec<char>> {
    let mut vec: Vec<Vec<char>> = vec![];
    let mut start = usize::MIN;
    while start < text.len() {
      let text          = &text[start..];
      let mut w         = 0;
      let mut max_width = 0;
      while w < width && max_width < text.len() {
        w         += &text[max_width].width().unwrap_or(0);
        max_width += 1;
      }
      let line: Vec<char> = {
        if text.len() <= max_width {
          text.to_vec()
        } else {
          // search for first whitespace from right
          let s: Vec<&char> = text[..max_width]
            .iter().rev().skip_while(|c| !c.is_whitespace()).collect();
          // no space found, return whole slice
          if s.len() == 0 {
            text[..max_width].iter().copied().collect()
          // space found, return up to that space
          } else {
            s.into_iter().rev().copied().collect()
          }
        }
      };
      start += line.len();
      vec.push(line);
    }
    vec
  }
  // get owned chars
  pub fn print(&self, width: usize) -> Vec<Vec<char>> {
    let text: Vec<char> = self.text.chars().collect();
    if text.len() == 0 {
      vec![vec![' ']]
    } else if self.wrap {
      Self::wrap_text(text, width)
    } else {
      vec![text]
    }
  }
  // get indexed owned chars
  pub fn print_vec(vec: &Vec<Self>, width: usize) -> Vec<(usize, Vec<char>)> {
    vec.iter().enumerate().flat_map(
      |(idx, styled)| 
        styled.print(width).into_iter().map(move |text| (idx, text))
      )
      .collect()
  }
}
#[derive(Clone, Debug, Default)]
pub struct ShiftedTextLine {
  pub idx:  usize,
  pub text: TextLine,
}
impl ShiftedTextLine {
  pub fn new(idx: usize, text: TextLine) -> Self {
    Self {idx, text}
  }
}
impl UnitCursor for ShiftedTextLine {
  type Unit = char;
  fn units(&self) -> &Vec<char> {
    &self.text.text
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.text.head
  }
  fn head(&self) -> usize {
    self.text.head
  }
  fn max_head(&self) -> usize {
    self.text.text.len().saturating_sub(1)
  }
}
impl Deref for ShiftedTextLine {
  type Target = TextLine;
  fn deref(&self) -> &Self::Target {
    &self.text
  }
}
impl DerefMut for ShiftedTextLine {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.text
  }
}
#[derive(Clone, Debug, Default)]
pub struct StyledTextPlane {
  pub text:   Vec<ShiftedTextLine>, 
  pub source: Vec<StyledText>,
  pub head:   usize,
  pub pref_x: usize,
}
impl UnitCursor for StyledTextPlane {
  type Unit = ShiftedTextLine;
  fn units(&self) -> &Vec<ShiftedTextLine> {
    &self.text
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
  fn max_head(&self) -> usize {
    self.text.len().saturating_sub(1)
  }
}
impl StyledTextPlane {
  pub fn new(source: Vec<StyledText>, width: u16) -> Self {
    let text = StyledText::print_vec(&source, usize::from(width))
      .into_iter()
      .map(|(idx, text)| ShiftedTextLine::new(idx, TextLine::from(text)));
    Self {
      source,
      head:   0, 
      pref_x: 0, 
      text:   text.collect(), 
    }
  }
  pub fn get_source_idx(&self) -> usize {
    self.current().idx
  }
  pub fn get_source(&self) -> String {
    self.source[self.current().idx].text.clone()
  }
  pub fn get_idx(&self) -> usize {
    let x_head = self.current().head();
    self.text[..self.head()]
      .iter()
      .map(|line| line.units().len().max(1))
      .chain(std::iter::once(x_head))
      .sum()
  }
  fn set_idx(&mut self, idx: usize) {
    self.start();
    // this guard responds to an error only encountered on windows
    if self.text.len() > 0 {
      self.text[self.head].start();
    }
    self.right(idx);
  }
  pub fn restyle(&mut self, source: Vec<StyledText>, width: u16) {
    self.source = source;
    let idx   = self.get_idx();
    self.text = StyledText::print_vec(&self.source, usize::from(width))
      .into_iter()
      .map(|(idx, text)| ShiftedTextLine::new(idx, TextLine::from(text)))
      .collect();
    self.set_idx(idx);
  }
  pub fn resize(&mut self, width: u16) {
    let idx   = self.get_idx();
    self.text = StyledText::print_vec(&self.source, usize::from(width))
      .into_iter()
      .map(|(idx, text)| ShiftedTextLine::new(idx, TextLine::from(text)))
      .collect();
    self.set_idx(idx);
  }
  pub fn up(&mut self, delta: usize) -> bool {
    if self.backward(delta) != delta {
      self.text[self.head].fit(self.pref_x);
      true
    } else {false}
  }
  pub fn down(&mut self, delta: usize) -> bool {
    if self.forward(delta) != delta {
      self.text[self.head].fit(self.pref_x);
      true
    } else {false}
  }
  pub fn left(&mut self, delta: usize) -> usize {
    if self.text.len() == 0 {return delta}
    let remainder = self.text[self.head].backward(delta);
    if remainder == 0 {
      self.pref_x = self.text[self.head].head();
      0
    } else if self.backward(1) == 0 {
      self.text[self.head].end();
      self.left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
  pub fn right(&mut self, delta: usize) -> usize {
    if self.text.len() == 0 {return delta}
    let remainder = self.text[self.head].forward(delta);
    if remainder == 0 {
      self.pref_x = self.text[self.head].head();
      0
    } else if self.forward(1) == 0 {
      self.text[self.head].start();
      self.right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}
