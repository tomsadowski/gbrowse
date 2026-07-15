// src/layout.rs

use crate::{
  SystemParams,
  TextParams, 
  Rect,
  Cursor, 
  Style, 
  Dialog,
  CursorVec,
  GemText,
  GemTag,
  Page,
  constants::*,
};



pub trait BuildView<T> where T: View {
  fn build(self, _: &Rect) -> T;
}


pub trait GetHeight {
  fn get_height(&self) -> u16;
}


impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 {
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}


pub trait View: GetHeight {
  fn draw(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>;


  fn resize(&mut self, rect: &Rect);
}


pub fn get_heights(vec: &[impl GetHeight]) -> u16 {
  vec.iter().map(|v| v.get_height()).sum()
}


pub fn resize_views(rect: &Rect, views: Vec<&mut impl View>) {
  let mut rect = rect.clone();
  for v in views {
    v.resize(&rect);
    rect = rect.shift_north((v.get_height() as i16) * -1);
  }
}


pub fn build_views<T: View>(rect: &Rect, params: Vec<impl BuildView<T>>) 
  -> Vec<T>
{
  let mut rect = rect.clone();
  let mut views: Vec<T> = vec![];
  for param in params {
    let view = param.build(&rect);
    rect = rect.shift_north((view.get_height() as i16) * -1);
    views.push(view);
  }
  views
}
