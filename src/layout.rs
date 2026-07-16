// src/layout.rs

use crate::{
  Rect,
};



pub trait BuildView<T> where T: Resize {
  fn build(self, _: &Rect) -> T;
}


pub trait GetHeight {
  fn get_height(&self) -> u16;
}


pub trait Draw {
  fn draw(&self, _: &mut impl std::io::Write) -> std::io::Result<()>;
}


pub trait Resize {
  fn resize(&mut self, rect: &Rect);
}


impl<T> GetHeight for Vec<T> {
  fn get_height(&self) -> u16 {
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
  view: &impl GetHeight, 
  w: &mut impl std::io::Write,
) -> std::io::Result<()> 
{
  let vheight = view.get_height().min(rect.height());
  rect
    .shift_north((vheight.saturating_sub(1) as i16) * -1)
    .draw(w)?;
  Ok(())
}


pub fn get_heights(rect: &Rect, views: &[impl GetHeight]) -> Vec<u16> {
  let mut vec: Vec<u16> = vec![];
  let mut rect = rect.clone();
  for v in views {
    let vheight = v.get_height().min(rect.height());
    rect = rect.shift_north((vheight as i16) * -1);
    vec.push(rect.h);
  }
  vec
}


pub fn resize_views<T: Resize + GetHeight>(
  rect: &Rect, views: Vec<&mut T>
) {
  let mut rect = rect.clone();
  for v in views {
    v.resize(&rect);
    let vheight = v.get_height().min(rect.height());
    rect = rect.shift_north((vheight as i16) * -1);
  }
}


pub fn build_views<T: Resize + GetHeight>(
  rect: &Rect, params: Vec<impl BuildView<T>>
) -> Vec<T> {
  let mut rect = rect.clone();
  let mut views: Vec<T> = vec![];
  for param in params {
    let v = param.build(&rect);
    let vheight = v.get_height().min(rect.height());
    rect = rect.shift_north((vheight as i16) * -1);
    views.push(v);
  }
  views
}
