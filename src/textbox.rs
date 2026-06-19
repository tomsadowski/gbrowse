// src/textbox.rs

use crate::{
  ViewPort, 
  Rect,
  Style, 
  TextStyle,
  Cursor,
  IndexedCursor, 
  ScreenCursor,
};


#[derive(Default)]
pub struct TextBox {
  pub view:   Rect,
  pub style:  Style,
  pub text:   Vec<String>,
  pub styles: Vec<TextStyle>,
  pub matrix: IndexedCursor<Cursor<char>>,
  pub cursor: ScreenCursor,
  pub pref_x: usize,
  pub write:  bool,
}

impl ViewPort for TextBox {
  fn get_view_port(&self) -> Rect {self.view}
}

impl From<Rect> for TextBox {
  fn from(view: Rect) -> Self {
    Self {
      write:  true,
      pref_x: 0,
      style:  Style::default(),
      text:   vec![],
      styles: vec![],
      cursor: ScreenCursor::from(&view), 
      matrix: IndexedCursor::default(),
      view:   view.get_view_port(),
    }
  }
}

impl TextBox {
  pub fn style<S: Into<Style> + Copy>(mut self, style: S) -> Self {
    self.style = style.into();
    self
  }

  pub fn editor(mut self) -> Self {
    self.matrix.make_editor_lines();
    self.cursor.update(&self.matrix);
    self
  }

  pub fn text(mut self, text: Vec<String>, styles: Vec<TextStyle>) -> Self {
    self.set_text(text, styles);
    self
  }

  pub fn set_text(&mut self, text: Vec<String>, styles: Vec<TextStyle>) {
    self.text = text;
    self.set_styles(styles);
  }

  pub fn set_styles(&mut self, styles: Vec<TextStyle>) {
    self.styles = styles;
    self.reset_matrix();
    self.cursor.update(&self.matrix);
    self.view = self.used_rect();
    self.cursor.resize(&self.matrix, &self.view);
    self.reset_state();
  }

  pub fn reset_matrix(&mut self) {
    let linear_head = self.matrix.get_linear_head();
    self.matrix = IndexedCursor::print_from(&self.view, &self.text, &self.styles);
    self.matrix.set_linear_head(linear_head);
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    let old_view = self.view;
    self.view    = view.get_view_port();
    if old_view.w != self.view.w {
      self.reset_matrix();
    }
    self.view = self.used_rect();
    self.cursor.resize(&self.matrix, &self.view);
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
    self.matrix.use_current(|c| c.to_string())
  }

  pub fn get_current_text(&self) -> String {
    self.text
      .get(self.matrix.head)
      .map(|t| t.to_string())
      .unwrap_or("empty".into())
  }

  pub fn get_current_index(&self) -> usize {
    self.matrix.get_current_index()
  }

  pub fn delete(&mut self) -> bool {
    if self.matrix
      .use_current_mut(|c| c.delete())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.matrix);
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.matrix
      .use_current_mut(|c| c.backspace())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.matrix);
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, ch: char) -> bool {
    if self.matrix
      .use_current_mut(|c| c.insert(ch))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.matrix);
      self.write = true;
      true
    } else {false}
  }

  pub fn move_left(&mut self, delta: usize) -> bool {
    if self.matrix.move_left(delta) == 0 {
      self.pref_x = self.matrix.use_current(|c| c.head).unwrap_or(0);
      self.write = self.cursor.update(&self.matrix);
      true
    } else {false}
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    if self.matrix.move_right(delta) == 0 {
      self.pref_x = self.matrix.use_current(|c| c.head).unwrap_or(0);
      self.write = self.cursor.update(&self.matrix);
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if self.matrix.move_down(delta) {
      self.matrix.use_current_mut(|c| c.fit(self.pref_x));
      self.write = self.cursor.update(&self.matrix);
      true
    } else {false}
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    if self.matrix.move_up(delta) {
      self.matrix.use_current_mut(|c| c.fit(self.pref_x));
      self.write = self.cursor.update(&self.matrix);
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
    let mut cursor = self.cursor.clone();
    let mut x = cursor.get_x_view().get_start();
    let mut y = cursor.get_y_view().get_start();
    w
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.matrix.get_view(cursor.get_y_view()) {
      w.queue(Style::from(
        *self.styles.get(*index).unwrap_or(&TextStyle::default())
      ))?;
      for c in line.get_weighted_view(cursor.get_x_view()) {
        w.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      w.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
      for _ in x..self.view.x_end() {
        w.queue(Print(' '))?;
      }
      x = self.view.x; y += 1; w.queue(MoveTo(x, y))?;
    }
    w.queue(SetAttribute(Attribute::Reset))?.queue(&self.style)?;
    w.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
