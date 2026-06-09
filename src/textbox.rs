// src/textbox.rs

use crate::{
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
  CursorPlane,
  StyledText, 
  TextPlane,
  ScreenCursor,
  Style, 
  Action, 
  ViewPort, 
  Rect,
};
use crossterm::{
  QueueableCommand, 
  cursor::{MoveTo}, 
  style::{Print, SetAttribute, Attribute},
};
use unicode_width::UnicodeWidthChar;
use std::io::Write;


pub struct TextBox<Y> {
  pub view:           Rect,
  pub cursor:         ScreenCursor,
  pub styled_text:    Vec<StyledText>,
  pub text_plane:     TextPlane<Y>,
  pub style:          Style,
  pub write:          bool,
  pub write_unused_x: bool,
  pub write_unused_y: bool,
}
impl<Y, V> From<V> for TextBox<Y> 
where 
  V:            ViewPort, 
  TextPlane<Y>: Default,
{
  fn from(view: V) -> Self {
    Self {
      write_unused_x: true,
      write_unused_y: true,
      write:          true,
      style:          Style::default(),
      styled_text:    vec![StyledText::default()],
      cursor:         ScreenCursor::from(&view), 
      text_plane:     TextPlane::default(),
      view:           view.get_view_port(),
    }
  }
}
impl<Y: From<Vec<char>>> TextBox<Y> {
  pub fn new<V, O, F>(view: V, origin: &Vec<O>, to_styled_text: F) -> Self 
  where 
    V: ViewPort,
    F: Fn(&O) -> StyledText,
  {
    let styled_text = origin.iter().map(|i| to_styled_text(i)).collect();
    Self {
      write_unused_x: true,
      write_unused_y: true,
      write:          true,
      style:          Style::default(),
      cursor:         ScreenCursor::from(&view), 
      text_plane:     TextPlane::new(&view, &styled_text),
      view:           view.get_view_port(),
      styled_text,
    }
  }

  pub fn input<I, F>(mut self, input: &Vec<I>, func: F) -> Self 
  where F: Fn(&I) -> StyledText,
  {
    self.styled_text = input.iter().map(|i| func(i)).collect();
    self.text_plane = TextPlane::new(&self.view, &self.styled_text);
    self
  }

  pub fn set_input<I, F>(&mut self, input: &Vec<I>, func: F)
  where F: Fn(&I) -> StyledText,
  {
    self.styled_text = input.iter().map(|i| func(i)).collect();
    self.text_plane = TextPlane::new(&self.view, &self.styled_text);
  }
}

impl<T> TextBox<T> {
  pub fn get_current_reference(&self) -> String {
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

  pub fn write_unused_x(mut self, write: bool) -> Self {
    self.write_unused_x = write;
    self
  }

  pub fn write_unused_y(mut self, write: bool) -> Self {
    self.write_unused_y = write;
    self
  }

  pub fn write_unused(mut self, write: bool) -> Self {
    self
      .write_unused_x(write)
      .write_unused_y(write)
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
impl<T, W> TextBox<T> 
where 
  TextPlane<T>: CursorPlane + UnitCursor<Unit = W>,
  T:            UnitCursor + From<Vec<char>>,
  W:            WeightedCursor,
{
  pub fn restyle<I, F>(&mut self, input: &Vec<I>, func: F) 
  where F: Fn(&I) -> StyledText,
  {
    let idx     = self.text_plane.get_index();
    self.styled_text = input.iter().map(|i| func(i)).collect();
    self.text_plane   = TextPlane::new(&self.view, &self.styled_text);
    self.text_plane.set_index(idx);
    self.reset_state();
  }

  pub fn resize<V: ViewPort>(&mut self, view: V) {
    let idx   = self.text_plane.get_index();
    self.view = view.get_view_port();
    self.text_plane = TextPlane::new(&view, &self.styled_text);
    self.cursor.resize(&self.text_plane, &self.view);
    self.text_plane.set_index(idx);
    self.reset_state();
  }
}
impl<T, W> TextBox<T> 
where 
  TextPlane<T>: CursorPlane + UnitCursor<Unit = W>,
  W:            WeightedCursor,
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
}
impl<Y, W> TextBox<Y> 
where 
  TextPlane<Y>: CursorPlane + UnitCursor<Unit = W>,
  W:            WeightedCursor + UnitCursorMut<Unit = char>,
{
  pub fn delete(&mut self) -> bool {
    if self.text_plane.use_current_mut(|c| c.delete()).unwrap_or(false) {
      self.cursor.update(&self.text_plane);
      self.write = true;
      true
    } else {false}
  }

  pub fn backspace(&mut self) -> bool {
    if self.text_plane.use_current_mut(|c| c.backspace()).unwrap_or(false) {
      self.cursor.update(&self.text_plane);
      self.write = true;
      true
    } else {false}
  }

  pub fn insert(&mut self, ch: char) -> bool {
    if self.text_plane.use_current_mut(|c| c.insert(ch)).unwrap_or(false) {
      self.cursor.update(&self.text_plane);
      self.write = true;
      true
    } else {false}
  }

  pub fn update_edit(&mut self, action: &Action) {
    match action {
      Action::Backspace => {self.backspace();}
      Action::Delete    => {self.delete();}
      Action::Insert(c) => {self.insert(*c);}
      Action::MoveLeft  => {self.move_left(1);}
      Action::MoveRight => {self.move_right(1);}
      _ => {}
    }
  }
}
impl<T, C> TextBox<T> 
where 
  TextPlane<T>: CursorPlane,
  T:            WeightedCursor + UnitCursor<Unit = C>,
  C:            std::fmt::Display + UnicodeWidthChar + Copy,
{
  pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    if self.write {
      self.write_all(writer)?;
    }
    Ok(())
  }

  pub fn write_all<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
    let mut x = self.view.x;
    let mut y = self.view.y;
    writer
      .queue(MoveTo(x, y))?
      .queue(SetAttribute(Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.text_plane.get_view(
        self.cursor.get_y_scroll(), 
        self.view.h.into()
      ) 
    {
      writer.queue(&self.styled_text[*index].style)?;
      for c in line.get_weighted_view(
          self.cursor.get_x_scroll(), 
          self.view.w.into()
        ) 
      {
        writer.queue(Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      if self.write_unused_x {
        writer
          .queue(SetAttribute(Attribute::Reset))?
          .queue(&self.style)?;
        for _ in x..self.view.x_end() {
          writer.queue(Print(' '))?;
        }
      }
      x = self.view.x;
      y += 1;
      writer.queue(MoveTo(x, y))?;
    }
    if self.write_unused_y {
      writer
        .queue(SetAttribute(Attribute::Reset))?
        .queue(&self.style)?;
      for _ in y..self.view.y_end() {
        for _ in self.view.x_range() {
          writer.queue(Print(' '))?;
        }
        x = self.view.x;
        y += 1;
        writer.queue(MoveTo(x, y))?;
      }
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
  }
}
