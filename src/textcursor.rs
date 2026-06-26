// src/textcursor.rs

use crate::{
  Rect,
  Style, 
  LineCursorView,
  Point, 
  ScreenCursor,
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

#[derive(Default)]
pub struct TextCursor {
  pub width:   u16,
  pub style:   Style,
  pub style_vec:  Vec<TextStyle>,
  pub indexes: Vec<usize>,
  pub string_vec:    Vec<String>,
  pub matrix:  Vec<Vec<char>>, 
  pub point:   Point,
}

impl From<&Rect> for TextCursor {
  fn from(rect: &Rect) -> Self {
    Self {
      style:   Style::default(),
      string_vec:    vec![],
      style_vec:  vec![],
      matrix:  vec![],
      indexes: vec![],
      point:   Point::default(),
      width:   rect.width(),
    }
  }
}

impl TextCursor {
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
    self.point.make_editor(&self.matrix);
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
    self.string_vec = text;
    self.set_styles(buffed_styles);
  }

  pub fn set_styles(&mut self, styles: Vec<TextStyle>) {
    self.style_vec = styles;
    self.reset_matrix();
  }

  pub fn reset_matrix(&mut self) {
    let linear_head = self.point.get_linear_head(&self.matrix);
    let width = usize::from(self.width);
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
    self.indexes = indexes;
    self.matrix  = matrix;
    self.point.set_linear_head(&self.matrix, linear_head);
  }

  pub fn resize(&mut self, rect: &Rect) {
    let new_width = rect.width();
    if self.width != new_width {
      self.width = new_width;
      self.reset_matrix();
    }
  }

  pub fn get_current_string(&self) -> Option<String> {
    self.matrix.get(*self.point.y).map(|c| c.iter().collect())
  }

  pub fn get_current_text(&self) -> String {
    self.string_vec
      .get(*self.point.y)
      .map(|t| t.to_string())
      .unwrap_or("empty".into())
  }

  pub fn delete(&mut self) -> bool {
    self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.delete(c))
      .unwrap_or(false) 
  }

  pub fn backspace(&mut self) -> bool {
    self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.backspace(c))
      .unwrap_or(false) 
  }

  pub fn insert(&mut self, ch: char) -> bool {
    self.matrix
      .get_mut(*self.point.y)
      .map(|c| self.point.x.insert(c, ch))
      .unwrap_or(false) 
  }

  pub fn move_left(&mut self, delta: usize) -> bool {
    self.point.move_x(&self.matrix, delta as isize * -1) == 0
  }

  pub fn move_right(&mut self, delta: usize) -> bool {
    self.point.move_x(&self.matrix, delta as isize) == 0
  }

  pub fn move_down(&mut self, delta: usize) -> bool {
    self.point.move_y(&self.matrix, delta as isize)
  }

  pub fn move_up(&mut self, delta: usize) -> bool {
    self.point.move_y(&self.matrix, delta as isize * -1)
  }

  pub fn draw<W: std::io::Write>(&self, screen: &ScreenCursor, w: &mut W) 
    -> std::io::Result<()> 
  {
    use crossterm::{QueueableCommand, cursor, style};
    use unicode_width::UnicodeWidthChar;
    let crate::Pos(mut x, mut y) = screen.pos();
    w
      .queue(cursor::MoveTo(x, y))?
      .queue(style::SetAttribute(style::Attribute::Reset))?
      .queue(&self.style)?;
    for (index, line) in self.get_view(screen.get_y_view()) {
      w.queue(Style::from(
        *self.style_vec.get(*index).unwrap_or(&TextStyle::default())
      ))?;
      for c in screen.get_x_view().get_weighted_view(line) {
        w.queue(style::Print(c))?;
        x += u16::try_from(c.width().unwrap_or(0)).unwrap();
      }
      w
        .queue(style::SetAttribute(style::Attribute::Reset))?
        .queue(&self.style)?;
      for _ in x..screen.get_rect().x_end() {
        w.queue(style::Print(' '))?;
      }
      x = screen.pos().x(); 
      y += 1; 
      w.queue(cursor::MoveTo(x, y))?;
    }
    w.queue(style::SetAttribute(style::Attribute::Reset))?;
    Ok(())
  }
}
