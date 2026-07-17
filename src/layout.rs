// src/layout.rs

use crate::{
  Rect,
};



pub trait BuildView<T> where T: Resize {
  fn build(self, _: &Rect) -> T;
}


pub trait GetMaxHeight {
  fn get_max_height(&self) -> u16;
}


pub trait GetDisplayHeight {
  fn get_display_height(&self) -> u16;
}


pub trait Draw {
  fn draw(&self, _: &mut impl std::io::Write) -> std::io::Result<()>;
}


pub trait Resize {
  fn resize(&mut self, _: &Rect);
}


impl<T> GetMaxHeight for Vec<T> {
  fn get_max_height(&self) -> u16 {
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}


impl Draw for Rect {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor, style};

    w
      .queue(cursor::MoveTo(self.x, self.y))?
      .queue(style::SetAttribute(style::Attribute::Reset))?;

    for y in self.y_range() {
      w.queue(cursor::MoveTo(self.x, y))?;
      for x in self.x_range() {
        w.queue(style::Print(' '))?;
      }
    }
    Ok(())
  }
}


pub fn fill(
  rect: &Rect, 
  view: &impl GetDisplayHeight, 
  w: &mut impl std::io::Write,
) -> std::io::Result<()> 
{
  let display_height = view.get_display_height().min(rect.height());
  rect
    .shift_north((display_height.saturating_sub(1) as i16) * -1)
    .draw(w)?;
  Ok(())
}



pub fn met_display_bounds(rect: &Rect, views: &Vec<&impl GetMaxHeight>) 
  -> Vec<Rect> 
{
    let mut remaining = rect.clone();
    let mut vec: Vec<Rect> = vec![];
    let mut views = views.iter();
    while let Some(view) = views.next() && remaining.h > 0 {
      let view_height = view.get_max_height().min(remaining.h);
      let mut current = remaining.clone();
      current.h = view_height;
      vec.push(current);
      remaining = remaining.shift_north((view_height as i16) * -1);
    }
    while let Some(v) = views.next() {
      vec.push(remaining);
    }
    vec
}

pub fn get_display_bounds(rect: &Rect, views: &Vec<&impl GetMaxHeight>) 
  -> Vec<Rect> 
{
    let mut remaining = rect.clone();
    let mut vec: Vec<Rect> = vec![];
    let mut views = views.iter();
    while let Some(view) = views.next() {
      if remaining.h > 0 {
        let view_height = view.get_max_height().min(remaining.h);
        let mut current = remaining.clone();
        current.h = view_height;
        vec.push(current);
        remaining = remaining.shift_north((view_height as i16) * -1);
      } else {
        vec.push(remaining);
      }
    }
    vec
}


pub fn resize_views<T: Resize + GetMaxHeight>(
  rect: &Rect, views: &mut Vec<&mut T>
) {
  let bounds = get_display_bounds(&rect, 
    &views.iter().map(|v| &**v).collect()
  );
  for (view, bound) in views.iter_mut().zip(bounds.iter()) {
    view.resize(&bound);
  }
}


pub fn build_views<T: Resize + GetDisplayHeight>(
  rect: &Rect, params: Vec<impl BuildView<T>>
) -> Vec<T> {
  let mut rect = rect.clone();
  let mut views: Vec<T> = vec![];
  for param in params {
    let v = param.build(&rect);
    let vheight = v.get_display_height().min(rect.height());
    rect = rect.shift_north((vheight as i16) * -1);
    views.push(v);
  }
  views
}
