// src/layout.rs

use crate::{
  Rect, Page, Dialog,
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


pub enum ViewType<'a, T> {
  Dialog(&'a Dialog), 
  Page(&'a Page<T>),
}


impl<'a, T> GetMaxHeight for ViewType<'a, T> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Page(page) => page.get_max_height(),
    }
  }
}


impl<'a, T> GetDisplayHeight for ViewType<'a, T> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Page(page) => page.get_display_height(),
    }
  }
}


impl<'a, T: std::fmt::Display> Draw for ViewType<'a, T> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Page(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}


pub enum ViewTypeMut<'a, T> {
  Dialog(&'a mut Dialog), 
  Page(&'a mut Page<T>),
}


impl<'a, T> GetDisplayHeight for ViewTypeMut<'a, T> {
  fn get_display_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_display_height(),
      Self::Page(page) => page.get_display_height(),
    }
  }
}


impl<'a, T> GetMaxHeight for ViewTypeMut<'a, T> {
  fn get_max_height(&self) -> u16 {
    match self {
      Self::Dialog(dialog) => dialog.get_max_height(),
      Self::Page(page) => page.get_max_height(),
    }
  }
}


impl<'a, T: std::fmt::Display> Draw for ViewTypeMut<'a, T> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Dialog(dialog) => {dialog.draw(w)?;}
      Self::Page(page) => {page.draw(w)?;}
    }
    Ok(())
  }
}


impl<'a, T: std::fmt::Display> Resize for ViewTypeMut<'a, T> {
  fn resize(&mut self, rect: &Rect) {
    match self {
      Self::Dialog(dialog) => dialog.resize(rect),
      Self::Page(page) => page.resize(rect),
    }
  }
}


impl<T: Draw> Draw for Option<T> {
  fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    if let Some(t) = self { t.draw(w)? } 
    Ok(())
  }
}


impl<T: GetMaxHeight> GetMaxHeight for Option<T> {
  fn get_max_height(&self) -> u16 {
    if let Some(t) = self { t.get_max_height() } else { 0 }
  }
}


impl<T: GetDisplayHeight> GetDisplayHeight for Option<T> {
  fn get_display_height(&self) -> u16 {
    if let Some(t) = self { t.get_display_height() } else { 0 }
  }
}


impl<T: Resize> Resize for Option<T> {
  fn resize(&mut self, rect: &Rect) {
    if let Some(t) = self { t.resize(rect) }
  }
}


impl<T> GetMaxHeight for Vec<T> {
  fn get_max_height(&self) -> u16 {
    u16::try_from(self.len()).unwrap_or(u16::MAX)
  }
}


pub fn fill(
  rect: &Rect, view: &impl GetDisplayHeight, w: &mut impl std::io::Write,
) -> std::io::Result<()> {
  rect
    .shift_north(
      view
        .get_display_height()
        .min(rect.h())
        .saturating_sub(1) as i16 * -1
    )
    .draw(w)?;
  Ok(())
}


pub fn get_display_bounds(
  rect: &Rect, views: &Vec<&impl GetMaxHeight>
) -> Vec<Rect> {
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


pub fn build_opt_views<T: Resize + GetDisplayHeight>(
  rect: &Rect, params: Vec<Option<impl BuildView<T>>>
) -> Vec<Option<T>> {
  let mut rect = rect.clone();
  let mut views: Vec<Option<T>> = vec![];
  for param in params {
    let v = param.map(|p| p.build(&rect));
    let vheight = v.get_display_height().min(rect.h());
    rect = rect.shift_north((vheight as i16) * -1);
    views.push(v);
  }
  views
}
