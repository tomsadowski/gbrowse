// src/textbox.rs

use crate::{
  ViewPort, 
  Rect,
  MatrixCursor, 
  MatrixCursorView,
  Style, 
  StyledText, 
  Action, 
  util,
};
use crossterm::{
  QueueableCommand, 
  cursor::{MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
use std::io::Write;

#[derive(Default)]
pub struct TextBox {
  pub view:           Rect,
  pub cursor:         MatrixCursorView,
  pub styled_text:    Vec<StyledText>,
  pub char_matrix:    MatrixCursor<char>,
  pub style:          Style,
  pub write:          bool,
  pub show_cursor:    bool,
}

impl From<Rect> for TextBox {
  fn from(view: Rect) -> Self {
    Self {
      write:          true,
      show_cursor:    false,
      style:          Style::default(),
      styled_text:    vec![StyledText::default()],
      cursor:         MatrixCursorView::from(&view), 
      char_matrix:    MatrixCursor::default(),
      view:           view.get_view_port(),
    }
  }
}

impl ViewPort for TextBox {
  fn get_view_port(&self) -> Rect {self.view}
}


impl TextBox {
  pub fn get_current_reference_string(&self) -> String {
    self.styled_text
      .get(self.char_matrix.get_current_reference_index())
      .map(|t| t.text.clone())
      .unwrap_or("empty".into())
  }

  pub fn get_current_reference_index(&self) -> usize {
    self.char_matrix.get_current_reference_index()
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
    if let Ok(h) = u16::try_from(self.char_matrix.len()) {
      self.view.cap_height(h)
    } else {
      self.view
    }
  }

  pub fn reset_state(&mut self) {
    self.write = true;
  }

  pub fn empty<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    writer
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for y in self.view.y_range() {
      for x in self.view.x_range() {
        writer.queue(MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }

  pub fn reference<R, F>(
    mut self, 
    reference:      &Vec<R>, 
    to_styled_text: F
  ) -> Self 
  where F: Fn(&R) -> StyledText,
  {
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.char_matrix = MatrixCursor::new(&self.view, &self.styled_text);
    self.cursor.update(&self.char_matrix);
    self
  }

  pub fn editor(mut self) -> Self {
    self.char_matrix.make_editor();
    self.cursor.update(&self.char_matrix);
    self
  }

  pub fn set_reference<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F)
  where F: Fn(&R) -> StyledText,
  {
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.char_matrix  = MatrixCursor::new(&self.view, &self.styled_text);
    self.cursor.update(&self.char_matrix);
  }

  pub fn restyle<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F) 
  where F: Fn(&R) -> StyledText,
  {
    let linear_head  = self.char_matrix.get_linear_head();
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.char_matrix  = MatrixCursor::new(&self.view, &self.styled_text);
    self.char_matrix.set_linear_head(linear_head);
    self.cursor.update(&self.char_matrix);
    self.reset_state();
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    let linear_head  = self.char_matrix.get_linear_head();
    self.view        = view.get_view_port();
    self.char_matrix = MatrixCursor::new(&view, &self.styled_text);
    self.char_matrix.set_linear_head(linear_head);
    self.cursor.resize(&self.char_matrix, &self.view);
    self.reset_state();
  }

  pub fn get_current_string(&self) -> Option<String> {
    self.char_matrix.use_current(|c| c.to_string())
  }

  pub fn delete(&mut self) -> bool {
    if self.char_matrix
      .use_current_mut(|c| c.delete())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.char_matrix);
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.char_matrix
      .use_current_mut(|c| c.backspace())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.char_matrix);
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, ch: char) -> bool {
    if self.char_matrix
      .use_current_mut(|c| c.insert(ch))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.char_matrix);
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
    if self.char_matrix.move_left(delta) == 0 {
      self.write = self.cursor.update(&self.char_matrix);
      true
    } else {false}
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    if self.char_matrix.move_right(delta) == 0 {
      self.write = self.cursor.update(&self.char_matrix);
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if self.char_matrix.move_down(delta) {
      self.write = self.cursor.update(&self.char_matrix);
      true
    } else {false}
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    if self.char_matrix.move_up(delta) {
      self.write = self.cursor.update(&self.char_matrix);
      true
    } else {false}
  }

  pub fn update(&mut self, action: &Action) {
    match action {
      Action::PageDown  => {self.move_down(usize::from(self.view.h));}
      Action::PageUp    => {self.move_up(usize::from(self.view.h));}
      Action::Bottom    => {self.move_down(self.char_matrix.len());}
      Action::Top       => {self.move_up(self.char_matrix.len());}
      Action::MoveDown  => {self.move_down(1);}
      Action::MoveUp    => {self.move_up(1);}
      Action::MoveLeft  => {self.move_left(1);}
      Action::MoveRight => {self.move_right(1);}
      _ => {}
    }
  }

  pub fn write<W: Write>(&self, writer: &mut W, overlay: u16) 
    -> std::io::Result<()> 
  {
    if self.write {
      self.write_all(writer, overlay)?;
    }
    Ok(())
  }

  pub fn write_all<W: Write>(&self, writer: &mut W, overlay: u16) 
    -> std::io::Result<()> 
  {
    let mut cursor = self.cursor.clone();
    if overlay > 0 {
      cursor.resize(
        &self.char_matrix, &self.view.crop_north(overlay)
      );
    }
    let mut x = cursor.get_x_view().get_start();
    let mut y = cursor.get_y_view().get_start();
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.char_matrix.get_view(cursor.get_y_view()) {
      writer.queue(&self.styled_text[*index].style)?;
      for c in util::get_weighted_view(
        &line, 
        |c| c.width().unwrap_or(0),
        cursor.get_x_view()
      ) {
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
//  for _ in y..self.view.y_end() {
//    for _ in self.view.x_range() {
//      writer.queue(Print(' '))?;
//    }
//    x = self.view.x; y += 1; writer.queue(MoveTo(x, y))?;
//  }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
