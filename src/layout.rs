// src/layout.rs

use crate::{
  ViewPort, 
  Rect,
  UnitCursor, 
  UnitCursorMut, 
  CursorPlane,
  TextPlane,
  ScreenCursor,
  Style, 
  StyledText, 
  Action, 
  TextBox,
  EditLine,
  TextLine,
};
use crossterm::{
  QueueableCommand, 
  cursor::{MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
use std::io::Write;


pub enum TextType {
  Edit(TextBox<EditLine>),
  Text(TextBox<TextLine>),
}

impl ViewPort for TextType {
  fn get_view_port(&self) -> Rect {
    match self {
      TextType::Edit(textbox) => textbox.view,
      TextType::Text(textbox) => textbox.view,
    }
  }
}

pub enum Orientation {
  Horizontal, Vertical,
}

pub struct Window {
  pub texttype:    TextType,
  pub orientation: Orientation,
}

pub struct Layout {
  pub view:    Rect,
  pub head:    usize,
  pub windows: Vec<Window>,
}

impl UnitCursor for Layout {
  type Unit = Window;
  fn get_units(&self) -> &Vec<Self::Unit> {
    &self.windows
  }
  fn get_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn get_head(&self) -> usize {
    self.head
  }
  fn get_max_head(&self) -> usize {
    self.windows.len().saturating_sub(1)
  }
}

impl UnitCursorMut for Layout {
  fn units_mut(&mut self) -> &mut Vec<Window> {
    &mut self.windows
  }
}

impl<V: ViewPort> From<V> for Layout {
  fn from(view: V) -> Self {
    Self {
      view:    view.get_view_port(),
      head:    0,
      windows: vec![],
    }
  }
}

impl Layout {
//fn get_weight(&self) -> usize {
//  match self.orientation {
//    Orientation::Horizontal => 
//      self.texttype.get_view_port().w.into(),
//    Orientation::Vertical => 
//      self.texttype.get_view_port().h.into(),
//  }
//}
}
