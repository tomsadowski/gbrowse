// src/layout.rs

use crate::{
  Rect,
};
use std::collections::HashMap;


pub enum ViewType {
  Peeper(Rect),
  DlgPrompt(Rect),
  DlgInput(Rect),
  Tab(Rect),
}

pub enum ViewName {
  Peeper,
  DlgPrompt,
  DlgInput,
  Tab,
}

pub struct LayoutBuilder {
  pub view:  Rect,
  pub views: Vec<Rect>
}

pub struct Layout {
  pub views: HashMap<String, Rect>,
}
