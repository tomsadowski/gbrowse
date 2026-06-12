// src/textbox.rs

use crate::{
  ViewPort, 
  Rect,
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
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


pub enum Window {
  Edit(TextBox<EditLine>),
  Text(TextBox<TextLine>),
}

pub struct Layout {
  pub view:           Rect,
  pub head:           usize,
  pub views:          Vec<Window>,
  pub write:          bool,
}
