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



pub trait BuildView<T> {
  fn build(_: &Rect) -> T;
}


pub trait View {
  fn get_height(&self) -> u16;


  fn draw(&self, writer: &mut std::io::Stdout) -> std::io::Result<()>;


  fn rebuild(&mut self, rect: &Rect);

  // dont rewrap, only point_view changes
  fn resize(&mut self, rect: &Rect);
}


pub fn resize_views(rect: &Rect, views: &mut Vec<impl View>) {
  let mut rect = rect.clone();
  for v in views {
    v.resize(&rect);
    rect = rect.shift_north((v.get_height() as i16) * -1);
  }
}


pub fn build_views<T>(rect: &Rect, params: Vec<impl BuildView<T>>) 
  -> Vec<T>
{
  let mut rect = rect.clone();
  let views: Vec<T> = vec![];
  for p in params {
    let v = p.build(&rect);
    rect = rect.shift_north((v.get_height() as i16) * -1);
    views.push(v);
  }
  views
}
