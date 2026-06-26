// src/layout.rs

use crate::{
  Rect, 
  GetRect,
  Frame,
  ScreenCursor,
  TextCursor,
};
use std::{
  collections::HashMap,
  rc::Rc,
};


pub trait GetHeight { fn get_height(&self) -> u16; }

impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 { 
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}

pub const TAB: u8 = 0;
pub const DLG: u8 = 1;
pub const MSG: u8 = 2;

pub enum Max {
  Size(u16),
  Fill,
}


pub enum View {
  Layout(Rc<Layout>),
  FramedLayout(Rc<Layout>, Frame),
  TextCursor(ScreenCursor, Rc<TextCursor>),
  FramedTextCursor(ScreenCursor, Rc<TextCursor>, Frame),
}

impl View {
  pub fn layout() {
  }

  pub fn framed_layout() {
  }

  pub fn text_cursor() {
  }

  pub fn framed_text_cursor() {
  }
}

pub struct ViewParams {
  max: Max,
  view: View,
}

impl ViewParams {
  pub fn layout() {
  }

  pub fn framed_layout() {
  }

  pub fn text_cursor() {
  }

  pub fn framed_text_cursor() {
  }
}


pub struct Layout {
  pub rect: Rect,
  pub views: HashMap<u8, ViewParams>,
}

impl<T: GetRect> From<&T> for Layout {
  fn from(t: &T) -> Self {
    Self {
      rect: t.get_rect(),
      views: HashMap::default(),
    }
  }
}

impl Layout {
  // add
  pub fn add(&mut self, handle: u8, params: ViewParams) {
  }
  // remove
  pub fn remove(&mut self, handle: u8) {
  }
}
