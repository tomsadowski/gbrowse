// src/page.rs

use crate::{
  Rect,
  Style, 
  CursorView,
  Point, 
  PointView,
};


#[derive(Copy, Clone, Debug)]
pub struct TextStyle {
  pub style: Style,
  pub wrap:  bool,
}

impl Default for TextStyle {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}

impl std::ops::Deref for TextStyle {
  type Target = Style;
  fn deref(&self) -> &Self::Target {&self.style}
}

impl TextStyle {
  // split at spaces within width and split at lines
  pub fn print(&self, width: usize, text: &str) -> Vec<Vec<char>> {
    if text.len() == 0 {
      vec![vec![]]
    } else if self.wrap {
      text
        .lines()
        .flat_map(|line| crate::util::get_wrapped_text(line, width))
        .collect()
    } else {
      text
        .lines()
        .map(|line| line.chars().collect())
        .collect()
    }
  }
}

#[derive(Default, Debug)]
pub struct PageParams {
  pub edit:       bool,
  pub style:      Style,
  pub style_vec:  Vec<TextStyle>,
  pub string_vec: Vec<String>,
}

impl PageParams {
  pub fn init() -> Self { Self::default() }
  pub fn set_edit(&mut self, b: bool) { self.edit = b; }
  pub fn edit(mut self, b: bool) -> Self { self.set_edit(b); self }
  pub fn set_style<S: Into<Style> + Copy>(&mut self, style: S) {
    self.style = style.into();
  }
  pub fn with_style<S: Into<Style> + Copy>(mut self, style: S) -> Self {
    self.set_style(style);
    self
  }
  pub fn set_text_styles(&mut self, styles: Vec<TextStyle>) 
    -> Result<(), String> 
  {
    if styles.len() < self.string_vec.len() {
      Err("styles vec shorter than string vec".into())
    } else {
      self.style_vec = styles;
      Ok(())
    }
  }
  pub fn with_text_styles(mut self, styles: Vec<TextStyle>) 
    -> Result<Self, String> 
  {
    self.set_text_styles(styles).map(|_| self)
  }
  pub fn set_text<T>(&mut self, text: &[T])
  where T: std::fmt::Display,
  {
    self.string_vec = text.iter().map(|t| t.to_string()).collect();
    self.style_vec  = self.string_vec.iter()
      .map(|t| TextStyle::default()).collect();
  }
  pub fn with_text<T>(mut self, text: &[T]) -> Self 
  where T: std::fmt::Display,
  {
    self.set_text(text);
    self
  }
  pub fn set_styled_text<T, F>(&mut self, text: &[T], get_text_style: F)
  where T: std::fmt::Display,
        F: Fn(&T) -> TextStyle,
  {
    self.string_vec = text.iter().map(|t| t.to_string()).collect();
    self.style_vec  = text.iter().map(|t| get_text_style(t)).collect();
  }
  pub fn with_styled_text<T, F>(mut self, text: &[T], get_text_style: F) 
    -> Self 
  where T: std::fmt::Display,
        F: Fn(&T) -> TextStyle,
  {
    self.set_styled_text(text, get_text_style);
    self
  }
  pub fn build(&self, width: u16) -> Page {
    let width = usize::from(width);
    let (indexes, matrix): (Vec<usize>, Vec<Vec<char>>) = self.style_vec
      .iter()
      .zip(self.string_vec.iter())
      .enumerate()
      .flat_map(|(idx, (style, string))| 
        style
          .print(width, string)
          .into_iter()
          .map(move |text| (idx, text))
      ).unzip();
    let point = if self.edit {
      Point::init().editor(&matrix)
    } else {
      Point::init()
    };
    Page {
      point, 
      indexes, 
      matrix,
    }
  }
  pub fn get_string(&self, page: &Page) -> &str {
    self.string_vec
      .get(*page.point.y)
      .map(|s| s.as_str())
      .unwrap_or("empty")
  }
  pub fn draw(
    &self, 
    scroll:     &Page,
    point_view: &PointView, 
    writer:     &mut impl std::io::Write
  ) -> std::io::Result<()> {
    scroll.draw(&self.style, &self.style_vec, point_view, writer)
  }
}


#[derive(Default)]
pub struct Page {
  pub indexes:    Vec<usize>,
  pub matrix:     Vec<Vec<char>>, 
  pub point:      Point,
}

impl Page {
  pub fn build(source: &PageParams, width: u16) -> Self {
    source.build(width)
  }

  pub fn rebuild(&mut self, source: &PageParams, width: u16) {
    let linear_head = self.point.get_linear_head(&self.matrix);
    let scroll = source.build(width);
    self.indexes = scroll.indexes;
    self.matrix  = scroll.matrix;
    self.point.set_linear_head(&self.matrix, linear_head);
  }

  pub fn get_index(&self) -> usize {
    self.indexes.get(*self.point.y)
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }

  pub fn get_string(&self) -> Option<String> {
    self.matrix.get(*self.point.y).map(|c| c.iter().collect())
  }

  pub fn get_view(&self, axis: CursorView) -> Vec<(&usize, &Vec<char>)> {
    self.indexes
      .iter()
      .zip(self.matrix.iter())
      .skip(axis.scroll)
      .take(usize::from(axis.size))
      .collect()
  }

  pub fn delete(&mut self) -> bool {
    self.point.delete(&mut self.matrix)
  }

  pub fn backspace(&mut self) -> bool {
    self.point.backspace(&mut self.matrix)
  }

  pub fn insert(&mut self, ch: char) -> bool {
    self.point.insert(&mut self.matrix, ch)
  }

  pub fn move_left(&mut self, delta: usize) -> bool {
    self.point.move_left(&self.matrix, delta)
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    self.point.move_right(&self.matrix, delta)
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    self.point.move_down(&self.matrix, delta)
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    self.point.move_up(&self.matrix, delta)
  }

  pub fn draw(
    &self, 
    style:      &Style,
    style_vec:  &[TextStyle],
    point_view: &PointView, 
    writer:     &mut impl std::io::Write,
  ) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor, style};
    use unicode_width::UnicodeWidthChar;
    let (mut x, mut y) = point_view.pos().into();
    writer
      .queue(cursor::MoveTo(x, y))?
      .queue(style::SetAttribute(style::Attribute::Reset))?
      .queue(&style)?;
    for (index, line) in self.get_view(point_view.get_y_view()) {
      writer.queue(Style::from(
        *style_vec.get(*index).unwrap_or(&TextStyle::default())
      ))?;
      for (w, c) in point_view.get_x_view().get_weighted_view(line) {
        writer.queue(style::Print(c))?;
        x += w;
      }
      writer
        .queue(style::SetAttribute(style::Attribute::Reset))?
        .queue(&style)?;
      for _ in x..point_view.get_rect().x_end() {
        writer.queue(style::Print(' '))?;
      }
      x = point_view.pos().x(); 
      y += 1; 
      writer.queue(cursor::MoveTo(x, y))?;
    }
    writer.queue(style::SetAttribute(style::Attribute::Reset))?;
    Ok(())
  }
}
