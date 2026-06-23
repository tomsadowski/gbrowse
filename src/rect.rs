// src/rect.rs

#[derive(Clone, Copy, Default)]
pub struct Dim(pub u16, pub u16);

impl From<(u16, u16)> for Dim {
  fn from((w, h): (u16, u16)) -> Self { Self(w.into(), h.into()) }
}

impl Dim {
  pub fn w(&self) -> u16 { self.0 }
  pub fn h(&self) -> u16 { self.1 }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Pos(pub u16, pub u16);

impl From<(u16, u16)> for Pos {
  fn from((x, y): (u16, u16)) -> Self { Self(x, y) }
}

impl Pos {
  pub fn x(&self) -> u16 { self.0 }
  pub fn y(&self) -> u16 { self.1 }
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}

pub trait GetRect {
  fn get_rect(&self) -> Rect;
}

impl GetRect for Rect {
  fn get_rect(&self) -> Rect {self.clone()}
}

impl From<Dim> for Rect {
  fn from(d: Dim) -> Self {
    Self { x: 0, y: 0, w: d.w(), h: d.h() }
  }
}

impl Rect {
  pub fn dim(&self) -> Dim {
    (self.w, self.h).into()
  }

  pub fn pos(&self) -> Pos {
    (self.x, self.y).into()
  }

  pub fn with_dim(mut self, dim: Dim) -> Self {
    self.set_dim(dim);
    self
  }

  pub fn set_dim(&mut self, dim: Dim) {
    self.w = dim.w(); 
    self.h = dim.h();
  }

  pub fn with_pos(mut self, pos: Pos) -> Self {
    self.set_pos(pos);
    self
  }

  pub fn set_pos(&mut self, pos: Pos) {
    self.x = pos.x(); 
    self.y = pos.y();
  }

  pub fn append_below(&mut self, above: &Rect) {
    self.y = above.y + self.h
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

  pub fn a(&self) -> Pos {
    (self.x, self.y).into()
  }

  pub fn b(&self) -> Pos {
    (self.x_end().saturating_sub(1), self.y).into()
  }

  pub fn c(&self) -> Pos {
    (self.x, self.y_end().saturating_sub(1)).into()
  }

  pub fn d(&self) -> Pos {
    (self.x_end().saturating_sub(1), self.y_end().saturating_sub(1)).into()
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
