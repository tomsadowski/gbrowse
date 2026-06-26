// src/layout.rs

use crate::{
  Rect, 
  Frame,
  ScreenCursor,
  TextCursor,
};
use std::{
  collections::HashMap,
  rc::Rc,
};


pub trait GetHeight { 
  fn get_height(&self) -> u16; 
}

impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 { 
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}

impl GetHeight for TextCursor {
  fn get_height(&self) -> u16 {
    self.matrix.get_height()
  }
}

impl GetHeight for Frame {
  fn get_height(&self) -> u16 {
    self.border_rect.height()
  }
}

pub struct TextBox {
  pub max:    Option<u16>,
  pub rect:   Rect,
  pub frame:  Frame,
  pub screen: ScreenCursor,
  pub text:   Rc<TextCursor>,
}

impl From<TextCursor> for TextBox {
  fn from(textcursor: TextCursor) -> Self {
    Self {
      max:    None,
      rect:   Rect::default(),
      frame:  Frame::default(),
      screen: ScreenCursor::default(),
      text:   Rc::new(textcursor),
    }
  }
}

impl TextBox {
  pub fn size(&mut self) {
    self.frame.resize(self.rect);

  }
}

pub enum View {
  Layout(Rc<Layout>),
  TextBox(TextBox),
}

pub struct Layout {
  pub max:   Option<u16>,
  pub rect:  Rect,
  pub frame: Frame,
  pub views: HashMap<u8, View>,
}

impl From<&Rect> for Layout {
  fn from(rect: &Rect) -> Self {
    Self {
      max:   None,
      rect:  rect.clone(),
      frame: Frame::from(rect),
      views: HashMap::default(),
    }
  }
}

impl Layout {
  // add
  pub fn add(&mut self, handle: u8, view: View) {
  }
  // remove
  pub fn remove(&mut self, handle: u8) {
  }
}
