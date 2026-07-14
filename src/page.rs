// src/page.rs

use crate::{
  cursor::get_weighted_length,
  Rect,
  Style, 
  CursorView,
  Point, 
  PointView,
  PointMatrix,
};


#[derive(Clone, Debug)]
pub struct TextView {
  pub params: TextStyleParams,
  pub string: String,
  pub matrix: Vec<Vec<char>>,
}


impl TextView {
  pub fn update(&mut self, params: TextStyleParams, width: usize) {
    if self.params.wrap != params.wrap {
      self.params.style = params.style;
    } else {
      self.params = params;
      self.matrix = self.params.get_matrix(&self.string, width);
    }
  }

  pub fn update_width(&mut self, width: usize) {
    self.matrix = self.params.get_matrix(&self.string, width);
  }
}


#[derive(Copy, Clone, Debug)]
pub struct TextStyleParams {
  pub style: Style,
  pub wrap:  bool,
}


impl From<&TextView> for TextStyleParams {
  fn from(view: &TextView) -> Self {
    Self {
      style: view.params.style,
      wrap:  view.params.wrap,
    }
  }
}


impl Default for TextStyleParams {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}


impl std::ops::Deref for TextStyleParams {
  type Target = Style;
  fn deref(&self) -> &Self::Target {
    &self.style
  }
}


impl TextStyleParams {
  pub fn get_view(self, string: &str, width: usize) -> TextView {
    TextView {
      matrix: self.get_matrix(string, width),
      params: self,
      string: string.into(),
    }
  }


  // split at spaces within width and split at lines
  pub fn get_matrix(&self, string: &str, width: usize) -> Vec<Vec<char>> {
    if string.len() == 0 {
      vec![vec![]]
    } else if self.wrap {
      string
        .lines()
        .flat_map(|line| crate::util::get_wrapped_text(line, width))
        .collect()
    } else {
      string
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
  pub style_vec:  Vec<TextStyleParams>,
  pub string_vec: Vec<String>,
}


impl PageParams {
  pub fn init() -> Self {
    Self::default()
  }


  pub fn edit(mut self, b: bool) -> Self {
    self.edit = b; self 
  }


  pub fn set_style(&mut self, style: impl Into<Style>) {
    self.style = style.into();
  }


  pub fn with_style(mut self, style: impl Into<Style>) -> Self {
    self.set_style(style);
    self
  }


  pub fn set_text_styles(&mut self, styles: Vec<TextStyleParams>) 
    -> Result<(), String> 
  {
    if styles.len() < self.string_vec.len() {
      Err("styles vec shorter than string vec".into())
    } else {
      self.style_vec = styles;
      Ok(())
    }
  }


  pub fn with_text_styles(mut self, styles: Vec<TextStyleParams>) 
    -> Result<Self, String> 
  {
    self.set_text_styles(styles).map(|_| self)
  }


  pub fn set_text(&mut self, text: &[impl std::fmt::Display]) {
    self.string_vec = text.iter().map(|t| t.to_string()).collect();
    self.style_vec  = self.string_vec.iter()
      .map(|t| TextStyleParams::default()).collect();
  }


  pub fn with_text(mut self, text: &[impl std::fmt::Display]) -> Self {
    self.set_text(text);
    self
  }


  pub fn set_styled_text<T, F>(&mut self, text: &[T], get_text_style: F)
  where T: std::fmt::Display,
        F: Fn(&T) -> TextStyleParams,
  {
    self.string_vec = text.iter().map(|t| t.to_string()).collect();
    self.style_vec  = text.iter().map(|t| get_text_style(t)).collect();
  }


  pub fn with_styled_text<T, F>(mut self, text: &[T], get_text_style: F) 
    -> Self 
  where T: std::fmt::Display,
        F: Fn(&T) -> TextStyleParams,
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
          .get_matrix(string, width)
          .into_iter()
          .map(move |text| (idx, text))
      ).unzip();
    let matrix = if self.edit {
      PointMatrix::from(matrix).editor()
    } else {
      PointMatrix::from(matrix)
    };
    Page { indexes, matrix }
  }


  pub fn rebuild(&self, page: &mut Page, width: u16) {
    let linear_head = page.matrix.get_linear_head();
    page.matrix = PointMatrix::default();
    let new_page = self.build(width);
    page.matrix = new_page.matrix;
    page.matrix.set_linear_head(linear_head);
  }


  pub fn get_string(&self, page: &Page) -> &str {
    self.string_vec
      .get(*page.matrix.point.y)
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


impl PointMatrix<char> {
}


#[derive(Default)]
pub struct Page {
  pub indexes: Vec<usize>,
  pub matrix:  PointMatrix<char>, 
}


impl Page {
  pub fn rebuild(&mut self, source: &PageParams, width: u16) {
    let linear_head = self.matrix.get_linear_head();
    let page = source.build(width);
    self.indexes = page.indexes;
    self.matrix  = page.matrix;
    self.matrix.set_linear_head(linear_head);
  }


  pub fn get_index(&self) -> usize {
    self.indexes.get(*self.matrix.point.y)
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }


  pub fn get_string(&self) -> Option<String> {
    self.matrix.matrix.get(*self.matrix.point.y).map(|c| c.iter().collect())
  }


  pub fn get_view(&self, axis: CursorView) -> Vec<(&usize, &Vec<char>)> {
    self.indexes
      .iter()
      .zip(self.matrix.matrix.iter())
      .skip(axis.scroll)
      .take(usize::from(axis.size))
      .collect()
  }


  pub fn draw(
    &self, 
    style:      &Style,
    style_vec:  &[TextStyleParams],
    point_view: &PointView, 
    writer:     &mut impl std::io::Write,
  ) 
    -> std::io::Result<()> 
  {
    use crossterm::{QueueableCommand, cursor, style};
    use unicode_width::UnicodeWidthChar;

    let (mut x, mut y) = point_view.pos().into();

    writer
      .queue(cursor::MoveTo(x, y))?
      .queue(style::SetAttribute(style::Attribute::Reset))?
      .queue(&style)?;

    for (index, line) in self.get_view(point_view.get_y_view()) {
      writer.queue(Style::from(
        style_vec.get(*index).unwrap_or(&TextStyleParams::default())
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
