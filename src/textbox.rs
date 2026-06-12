// src/textbox.rs

use crate::{
  ViewPort, 
  Rect,
  UnitCursor, 
  UnitCursorMut, 
  CursorPlane,
  TextPlane,
  ScreenCursor,
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


pub struct TextBox<T> {
  pub view:           Rect,
  pub cursor:         ScreenCursor,
  pub styled_text:    Vec<StyledText>,
  pub text_plane:     TextPlane<T>,
  pub style:          Style,
  pub write:          bool,
  pub show_cursor:    bool,
}

impl<T> TextBox<T> {
  pub fn get_current_reference_string(&self) -> String {
    self.styled_text
      .get(self.text_plane.get_current_reference_index())
      .map(|t| t.text.clone())
      .unwrap_or("empty".into())
  }

  pub fn get_current_reference_index(&self) -> usize {
    self.text_plane.get_current_reference_index()
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
    if let Ok(h) = u16::try_from(self.text_plane.get_length()) {
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
}

impl<T, V> From<V> for TextBox<T> 
where TextPlane<T>: Default,
      V:            ViewPort, 
{
  fn from(view: V) -> Self {
    Self {
      write:          true,
      show_cursor:    false,
      style:          Style::default(),
      styled_text:    vec![StyledText::default()],
      cursor:         ScreenCursor::from(&view), 
      text_plane:     TextPlane::default(),
      view:           view.get_view_port(),
    }
  }
}

impl<T> TextBox<T> 
where TextPlane<T>: UnitCursor<Unit = T>,
      T:            UnitCursor<Unit = char> + From<Vec<char>>,
{
  pub fn reference<R, F>(
    mut self, 
    reference:      &Vec<R>, 
    to_styled_text: F
  ) -> Self 
  where F: Fn(&R) -> StyledText,
  {
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.text_plane  = TextPlane::new(&self.view, &self.styled_text);
    self.cursor.update(&self.text_plane);
    self
  }

  pub fn set_reference<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F)
  where F: Fn(&R) -> StyledText,
  {
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.text_plane  = TextPlane::new(&self.view, &self.styled_text);
    self.cursor.update(&self.text_plane);
  }

  pub fn restyle<R, F>(&mut self, reference: &Vec<R>, to_styled_text: F) 
  where F: Fn(&R) -> StyledText,
  {
    let linear_head  = self.text_plane.get_linear_head();
    self.styled_text = reference.iter().map(|i| to_styled_text(i)).collect();
    self.text_plane  = TextPlane::new(&self.view, &self.styled_text);
    self.text_plane.set_linear_head(linear_head);
    self.cursor.update(&self.text_plane);
    self.reset_state();
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    let linear_head = self.text_plane.get_linear_head();
    self.view       = view.get_view_port();
    self.text_plane = TextPlane::new(&view, &self.styled_text);
    self.text_plane.set_linear_head(linear_head);
    self.cursor.resize(&self.text_plane, &self.view);
    self.reset_state();
  }
}

impl<T> TextBox<T> 
where TextPlane<T>: UnitCursor<Unit = T>,
      T:            ToString,
{
  pub fn get_current_string(&self) -> Option<String> {
    self.text_plane.use_current(|c| c.to_string())
  }
}

impl<T> TextBox<T> 
where TextPlane<T>: UnitCursor<Unit = T>,
      T:            UnitCursorMut<Unit = char>,
{
  pub fn delete(&mut self) -> bool {
    if self.text_plane
      .use_current_mut(|c| c.delete())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.text_plane);
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.text_plane
      .use_current_mut(|c| c.backspace())
      .unwrap_or(false) 
    {
      self.cursor.update(&self.text_plane);
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, ch: char) -> bool {
    if self.text_plane
      .use_current_mut(|c| c.insert(ch))
      .unwrap_or(false) 
    {
      self.cursor.update(&self.text_plane);
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
}

impl<T> TextBox<T> 
where TextPlane<T>: UnitCursor<Unit = T>,
      T:            UnitCursor<Unit = char>,
{
  pub fn move_left(&mut self, delta: usize) -> bool {
    if self.text_plane.move_left(delta) == 0 {
      self.write = self.cursor.update(&self.text_plane);
      true
    } else {false}
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    if self.text_plane.move_right(delta) == 0 {
      self.write = self.cursor.update(&self.text_plane);
      true
    } else {false}
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    if self.text_plane.move_down(delta) {
      self.write = self.cursor.update(&self.text_plane);
      true
    } else {false}
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    if self.text_plane.move_up(delta) {
      self.write = self.cursor.update(&self.text_plane);
      true
    } else {false}
  }

  pub fn update(&mut self, action: &Action) {
    match action {
      Action::PageDown  => {self.move_down(usize::from(self.view.h));}
      Action::PageUp    => {self.move_up(usize::from(self.view.h));}
      Action::Bottom    => {self.move_down(self.text_plane.get_length());}
      Action::Top       => {self.move_up(self.text_plane.get_length());}
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
        &self.text_plane, &self.view.crop_north(overlay)
      );
    }
    let mut x = cursor.get_x_line().get_start();
    let mut y = cursor.get_y_line().get_start();
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.text_plane.get_view(cursor.get_y_line()) {
      writer.queue(&self.styled_text[*index].style)?;
      for c in util::get_weighted_view(
        &line.get_units(), 
        |c| c.width().unwrap_or(0),
        cursor.get_x_line()
      ) 
      {
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
    for _ in y..self.view.y_end() {
      for _ in self.view.x_range() {
        writer.queue(Print(' '))?;
      }
      x = self.view.x; y += 1; writer.queue(MoveTo(x, y))?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
