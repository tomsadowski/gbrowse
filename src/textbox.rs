// src/textbox.rs

use crate::{
  ViewPort, 
  Rect,
  Action, 
  Style, 
  StyledText, 
  Cursor,
  IndexedCursor, 
  MatrixCursorView,
  Draw,
};


#[derive(Default)]
pub struct TextBox<T> {
  pub view:           Rect,
  pub style:          Style,
  pub text:           Vec<(T, Style)>,
  pub matrix:         Cursor<Cursor<char>>,
  pub cursor:         MatrixCursorView,
  pub pref_x:         usize,
  pub write:          bool,
  pub show_cursor:    bool,
}

impl<T> ViewPort for TextBox<T> {
  fn get_view_port(&self) -> Rect {self.view}
}

impl<T> From<Rect> for TextBox<T> {
  fn from(view: Rect) -> Self {
    Self {
      write:          true,
      show_cursor:    false,
      pref_x:         0,
      style:          Style::default(),
      text:           vec![],
      cursor:         MatrixCursorView::from(&view), 
      matrix:         Cursor::default(),
      view:           view.get_view_port(),
    }
  }
}

impl<T: std::fmt::Display> TextBox<T> {
  pub fn get_current_display_ref(&self) -> String {
    self.text
      .get(self.matrix.head)
      .map(|t| t.0.to_string())
      .unwrap_or("empty".into())
  }

  pub fn get_current_reference_index(&self) -> usize {
    self.matrix.get_current_reference_index()
  }

  pub fn style<S>(mut self, style: S) -> Self 
  where S: Into<Style> + Copy
  {
    self.style = style.into();
    self
  }

  pub fn show_cursor(mut self, b: bool) -> Self {
    self.show_cursor = b;
    self
  }

  pub fn used_rect(&self) -> Rect {
    if let Ok(h) = u16::try_from(self.matrix.len()) {
      self.view.cap_height(h)
    } else {
      self.view
    }
  }

  pub fn reset_state(&mut self) {
    self.write = true;
  }

  pub fn reference<F>(
    mut self, 
    reference:      &Vec<T>, 
    get_style: F
  ) -> Self 
  where F: Fn(&T) -> Style,
  {
    self.text   = reference.iter().map(|i| get_style(i)).collect();
    self.matrix = IndexedCursor::new(&self.view, &self.text);
    self.cursor.update(&self.matrix);
    self
  }

  pub fn as_editor(mut self) -> Self {
    self.matrix.make_editor_lines();
    self.cursor.update(&self.matrix);
    self
  }

  pub fn set_reference<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F)
  where F: Fn(&R) -> StyledText,
  {
    self.text   = reference.iter().map(|i| to_styled_text(i)).collect();
    self.matrix = IndexedCursor::new(&self.view, &self.text);
    self.cursor.update(&self.matrix);
  }

  pub fn restyle<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F) 
  where F: Fn(&R) -> StyledText,
  {
    let linear_head = self.matrix.get_linear_head();
    self.text       = reference.iter().map(|i| to_styled_text(i)).collect();
    self.matrix     = IndexedCursor::new(&self.view, &self.text);
    self.matrix.set_linear_head(linear_head);
    self.cursor.update(&self.matrix);
    self.reset_state();
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    let linear_head = self.matrix.get_linear_head();
    self.view       = view.get_view_port();
    self.matrix     = IndexedCursor::new(&view, &self.text);
    self.matrix.set_linear_head(linear_head);
    self.cursor.resize(&self.matrix, &view);
    self.reset_state();
  }

  pub fn get_current_string(&self) -> Option<String> {
    self.matrix.use_current(|c| c.to_string())
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

  pub fn update_edit(&mut self, action: &Action) {
    match action {
      Action::PageDown  => {self.move_left(usize::from(self.view.w));}
      Action::PageUp    => {self.move_right(usize::from(self.view.w));}
      Action::Backspace => {self.backspace();}
      Action::Delete    => {self.delete();}
      Action::Insert(c) => {self.insert(*c);}
      Action::MoveLeft  => {self.move_left(1);}
      Action::MoveRight => {self.move_right(1);}
      Action::MoveDown  => {self.move_down(1);}
      Action::MoveUp    => {self.move_up(1);}
      _ => {}
    }
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

  pub fn update(&mut self, action: &Action) {
    match action {
      Action::PageDown  => {self.move_down(usize::from(self.view.h));}
      Action::PageUp    => {self.move_up(usize::from(self.view.h));}
      Action::Bottom    => {self.move_down(self.matrix.len());}
      Action::Top       => {self.move_up(self.matrix.len());}
      Action::MoveDown  => {self.move_down(1);}
      Action::MoveUp    => {self.move_up(1);}
      Action::MoveLeft  => {self.move_left(1);}
      Action::MoveRight => {self.move_right(1);}
      _ => {}
    }
  }
}

impl Draw for TextBox {
  fn draw<W: std::io::Write>(&self, writer: &mut W) 
    -> std::io::Result<()> 
  {
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
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.matrix.get_view(cursor.get_y_view()) {
      writer.queue(&self.text[*index].style)?;
      for c in line.get_weighted_view(cursor.get_x_view()) {
        writer.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      writer
        .queue(SetAttribute(Attribute::Reset))?
        .queue(&self.style)?;
      for _ in x..self.view.x_end() {
        writer.queue(Print(' '))?;
      }
      x = self.view.x; y += 1; writer.queue(MoveTo(x, y))?;
    }
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
