// src/rect.rs


#[derive(Clone, Copy, Default)]
pub struct Dim(pub u16, pub u16);

impl From<(u16, u16)> for Dim {
  fn from((w, h): (u16, u16)) -> Self { 
    Self(w.into(), h.into()) 
  }
}

impl Dim {
  pub fn w(&self) -> u16 { self.0 }
  pub fn h(&self) -> u16 { self.1 }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Pos(pub u16, pub u16);

impl From<(u16, u16)> for Pos {
  fn from((x, y): (u16, u16)) -> Self { 
    Self(x, y) 
  }
}

impl Pos {
  pub fn x(&self) -> u16 { self.0 }
  pub fn y(&self) -> u16 { self.1 }
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
  x: u16,
  y: u16,
  w: u16,
  h: u16,
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

impl From<Pos> for Rect {
  fn from(p: Pos) -> Self {
    Self { x: p.x(), y: p.y(), w: 0, h: 0 }
  }
}

impl Rect {
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

  pub fn shift_north(&self, idelta: i16) -> Self {
    let mut rect = self.clone();
    rect.y = (rect.y as i16).saturating_add(idelta) as u16;
    rect.h = (rect.h as i16).saturating_sub(idelta) as u16;
    rect
  }

  pub fn shift_south(&self, idelta: i16) -> Self {
    let mut rect = self.clone();
    rect.h = (rect.h as i16).saturating_sub(idelta) as u16;
    rect
  }

  pub fn shift_east(&self, idelta: i16) -> Self {
    let mut rect = self.clone();
    rect.w = (rect.w as i16).saturating_sub(idelta) as u16;
    rect
  }

  pub fn shift_west(&self, idelta: i16) -> Self {
    let mut rect = self.clone();
    rect.x = (rect.x as i16).saturating_add(idelta) as u16;
    rect.w = (rect.w as i16).saturating_sub(idelta) as u16;
    rect
  }

  pub fn shift_y(&self, idelta: i16) -> Self {
    self.shift_north(idelta).shift_south(idelta)
  }

  pub fn shift_x(&self, idelta: i16) -> Self {
    self.shift_east(idelta).shift_west(idelta)
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

  pub fn x(&self)      -> u16 { self.pos().x() }
  pub fn y(&self)      -> u16 { self.pos().y() }
  pub fn width(&self)  -> u16 { self.dim().w() }
  pub fn height(&self) -> u16 { self.dim().h() }
  pub fn dim(&self)    -> Dim { (self.w, self.h).into() }
  pub fn pos(&self)    -> Pos { (self.x, self.y).into() }
  pub fn x_end(&self)  -> u16 { self.x + self.w }
  pub fn y_end(&self)  -> u16 { self.y + self.h }

  pub fn a(&self) -> Pos {
    (self.x, self.y).into()
  }

  pub fn b(&self) -> Pos {
    (self.x_end().saturating_sub(1), 
     self.y).into()
  }

  pub fn c(&self) -> Pos {
    (self.x, 
     self.y_end().saturating_sub(1)).into()
  }

  pub fn d(&self) -> Pos {
    (self.x_end().saturating_sub(1), 
     self.y_end().saturating_sub(1)).into()
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
