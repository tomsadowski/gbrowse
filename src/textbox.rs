// src/textbox.rs

use crate::{
  GetRect, 
  Rect,
  Style, 
  TextStyle,
  LineCursorView,
  Point, 
  ScreenCursor,
};


#[derive(Default)]
pub struct TextBox {
  pub view:    Rect,
  pub style:   Style,
  pub text:    Vec<String>,
  pub styles:  Vec<TextStyle>,
  pub indexes: Vec<usize>,
  pub matrix:  Vec<Vec<char>>, 
  pub point:   Point,
  pub cursor:  ScreenCursor,
  pub write:   bool,
}

impl GetRect for TextBox {
  fn get_rect(&self) -> Rect {self.view}
}

impl From<Rect> for TextBox {
  fn from(view: Rect) -> Self {
    Self {
      write:   true,
      style:   Style::default(),
      text:    vec![],
      styles:  vec![],
      matrix:  vec![],
      indexes: vec![],
      point:   Point::default(),
      cursor:  ScreenCursor::from(&view), 
      view:    view.get_rect(),
    }
  }
}

impl TextBox {
  pub fn get_current_index(&self) -> usize {
    self.indexes.get(*self.point.y)
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }

  pub fn get_view(&self, axis: LineCursorView) -> Vec<(&usize, &Vec<char>)> {
    self.indexes
      .iter()
      .zip(self.matrix.iter())
      .skip(axis.scroll)
      .take(usize::from(axis.size))
      .collect()
  }

  pub fn style<S: Into<Style> + Copy>(mut self, style: S) -> Self {
    self.style = style.into();
    self
  }

  pub fn editor(mut self) -> Self {
    self.point.make_editor();
    self.cursor.update(&self.point);
    self
  }

  pub fn text(mut self, text: Vec<String>, styles: Vec<TextStyle>) -> Self {
    self.set_text(text, styles);
    self
  }

  pub fn set_text(&mut self, text: Vec<String>, styles: Vec<TextStyle>) {
    let buffed_styles: Vec<_> = if styles.len() < text.len() {
      text.iter().map(|_| TextStyle::default()).collect()
    } else {
      styles
    };
    self.text = text;
    self.set_styles(buffed_styles);
  }

  pub fn set_styles(&mut self, styles: Vec<TextStyle>) {
    self.styles = styles;
    self.reset_matrix();
    self.cursor.update(&self.point);
    self.view = self.used_rect();
    self.cursor.resize(&self.point, &self.view);
    self.reset_state();
  }

  pub fn reset_matrix(&mut self) {
    let linear_head = self.point.get_linear_head(&self.matrix);
    let width = usize::from(self.view.w);
    let (indexes, matrix): (Vec<usize>, Vec<Vec<char>>) = self.styles
      .iter()
      .zip(self.text.iter())
      .enumerate()
      .flat_map(
      |(idx, (style, text))| 
        style
          .print(width, text)
          .into_iter()
          .map(move |text| (idx, text))
      ).unzip();
    self.indexes = indexes;
    self.matrix  = matrix;
    self.point.set_linear_head(&self.matrix, linear_head);
  }

  pub fn resize<V: GetRect>(&mut self, view: V) {
    let old_view = self.view;
    self.view    = view.get_rect();
    if old_view.w != self.view.w {
      self.reset_matrix();
    }
    self.view = self.used_rect();
    self.cursor.resize(&self.point, &self.view);
    self.reset_state();
  }

  pub fn reset_state(&mut self) {
    self.write = true;
  }

  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.matrix.len()) {
      self.view.cap_height(h)
    } else {self.view}
  }

  pub fn get_current_string(&self) -> Option<String> {
    self.matrix.get(*self.point.y).map(|c| c.iter().collect())
  }

  pub fn get_current_text(&self) -> String {
    self.text
      .get(*self.point.y)
      .map(|t| t.to_string())
      .unwrap_or("empty".into())
  }

  pub fn delete(&mut self) -> bool {
    if self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.delete(c))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.point);
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.backspace(c))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.point);
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, ch: char) -> bool {
    if self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.insert(c, ch))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.point);
      self.write = true;
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> bool {
    if self.point.move_x(&self.matrix, delta as isize * -1) == 0 {
      self.write = self.cursor.update(&self.point);
      true
    } else {false}
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    if self.point.move_x(&self.matrix, delta as isize) == 0 {
      self.write = self.cursor.update(&self.point);
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if self.point.move_y(&self.matrix, delta as isize) {
      self.write = self.cursor.update(&self.point);
      true
    } else {false}
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    if self.point.move_y(&self.matrix, delta as isize * -1) {
      self.write = self.cursor.update(&self.point);
      true
    } else {false}
  }
}

impl crate::Draw for TextBox {
  fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    if !self.write {return Ok(())}
    use crossterm::{
      QueueableCommand, 
      cursor::{MoveTo}, 
      style::{Print, SetAttribute, Attribute},
    };
    use unicode_width::UnicodeWidthChar;
    let crate::Pos(mut x, mut y) = self.view.pos();
    w
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.get_view(self.cursor.get_y_view()) {
      w.queue(Style::from(
        *self.styles.get(*index).unwrap_or(&TextStyle::default())
      ))?;
      for c in self.cursor.get_x_view().get_weighted_view(line) {
        w.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      w.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
      for _ in x..self.view.x_end() {
        w.queue(Print(' '))?;
      }
      x = self.view.x; 
      y += 1; 
      w.queue(MoveTo(x, y))?;
    }
    w.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    w.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
