// src/style.rs

use crate::{
  util,
  Style,
  TextStyle,
};


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
      vec![vec![]]
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
