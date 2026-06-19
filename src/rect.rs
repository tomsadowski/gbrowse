// src/rect.rs


pub trait ViewPort {
  fn get_view_port(&self) -> Rect;
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}

impl ViewPort for Rect {
  fn get_view_port(&self) -> Rect {self.clone()}
}

impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    Self {x: 0, y: 0, w, h}
  }

  pub fn crop_north(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta * 2 < rect.h {
      rect.y += delta;
      rect.h -= delta;
    }
    rect
  }

  pub fn crop_south(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta < rect.h {
      rect.h -= delta;
    }
    rect
  }

  pub fn crop_east(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta < rect.w {
      rect.w -= delta
    }
    rect
  }

  pub fn crop_west(&self, delta: u16) -> Self {
    let mut rect = self.clone();
    if delta * 2 < rect.w {
      rect.x += delta;
      rect.w -= delta;
    }
    rect
  }

  pub fn crop_y(&self, delta: u16) -> Self {
    self.crop_north(delta).crop_south(delta)
  }

  pub fn crop_x(&self, delta: u16) -> Self {
    self.crop_east(delta).crop_west(delta)
  }

  pub fn get_compliment(&self, container: &Self) -> Self {
    self.clone()
  }

  pub fn row(&self, y: u16) -> Self {
    Self {
      x: self.x, 
      y: y, 
      w: self.w, 
      h: 1
    }
  }

  pub fn top_row(&self) -> Self {
    self.row(self.y)
  }

  pub fn bottom_row(&self) -> Self {
    self.row(self.y_end())
  }

  pub fn cap_height(&self, h: u16) -> Self {
    let mut rect = self.clone();
    rect.h = h.min(rect.h);
    rect
  }

  pub fn x_end(&self) -> u16 {self.x + self.w}

  pub fn y_end(&self) -> u16 {self.y + self.h}

  pub fn a(&self) -> (u16, u16) {
    (self.x, self.y)
  }

  pub fn b(&self) -> (u16, u16) {
    (self.x_end().saturating_sub(1), self.y)
  }

  pub fn c(&self) -> (u16, u16) {
    (self.x, self.y_end().saturating_sub(1))
  }

  pub fn d(&self) -> (u16, u16) {
    (self.x_end().saturating_sub(1), 
     self.y_end().saturating_sub(1))
  }

  pub fn x_range(&self) -> std::ops::Range<u16> {
    std::ops::Range {
      start: self.x, 
      end:   self.x_end()
    }
  }

  pub fn y_range(&self) -> std::ops::Range<u16> {
    std::ops::Range {
      start: self.y, 
      end:   self.y_end()
    }
  }

  pub fn resize(&mut self, w: u16, h: u16) {
    self.w = w; 
    self.h = h;
  }
}
