// src/layout.rs

use crate::{
    Rect, 
    Page, 
    Dialog,
};



pub trait BuildView<T> where T: View {
    fn build(self, _: &Rect) -> T;
}

pub trait View {
    fn resize(&mut self, _: &Rect) {}
    fn draw(&self, _: &mut dyn std::io::Write) -> std::io::Result<()> {Ok(())}
    fn get_display_height(&self) -> u16 {0}
    fn get_max_height(&self) -> u16 {0}
}
impl<T> View for Vec<T> {
    fn get_max_height(&self) -> u16 {
        u16::try_from(self.len()).unwrap_or(u16::MAX)
    }
}

pub fn fill(
    rect: &Rect, view: &dyn View, w: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    rect
        .shift_north(
            view
                .get_display_height()
                .min(rect.h())
                .saturating_sub(1) as i16 * -1
        )
        .draw(w)
}

pub fn get_display_bounds(
    rect: &Rect, views: &Vec<&dyn View>
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

pub fn resize_views(
    rect: &Rect, views: &mut Vec<&mut dyn View>
) {
    let bounds = get_display_bounds(&rect, 
        &views.iter().map(|v| &**v).collect()
    );
    for (view, bound) in views.iter_mut().zip(bounds.iter()) {
        view.resize(&bound);
    }
}

//  pub fn build_opt_views<T: View>(
//      rect: &Rect, params: Vec<&dyn BuildView<T>>
//  ) -> Vec<T> {
//      let mut rect = rect.clone();
//      let mut views: Vec<_> = vec![];
//      for param in params {
//          let v = param.build(&rect);
//          let vheight = v.get_display_height().min(rect.h());
//          rect = rect.shift_north((vheight as i16) * -1);
//          views.push(v);
//      }
//      views
//  }
