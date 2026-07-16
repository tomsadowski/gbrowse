// src/page.rs

use crate::{
  cursor::get_weighted_length,
  Rect,
  Style, 
  CursorView,
  Point, 
  PointView,
  GetHeight,
  BuildView,
  Resize,
  PointMatrix,
  Draw,
};



#[derive(Copy, Clone, Debug)]
pub struct TextParams {
  pub style: Style,
  pub wrap:  bool,
}


impl From<&TextParams> for Style {
  fn from(t: &TextParams) -> Self {
    t.style
  }
}


impl Default for TextParams {
  fn default() -> Self {
    Self {
      style: Style::default(),
      wrap:  true,
    }
  }
}


impl TextParams {
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


pub fn print(
  width: usize, 
  styles: &Vec<TextParams>, 
  source: &Vec<impl std::fmt::Display>
) -> (Vec<usize>, Vec<Vec<char>>) {
  styles
    .iter()
    .zip(source.iter())
    .enumerate()
    .flat_map(|(idx, (style, source))| 
      style
        .get_matrix(&source.to_string(), width)
        .into_iter()
        .map(move |text| (idx, text))
    ).unzip()
}


#[derive(Debug)]
pub struct PageParams<T> {
  pub max: Option<u16>,
  pub draw_point: bool,
  pub edit: bool,
  pub style: Style,
  pub styles: Vec<TextParams>,
  pub source: Vec<T>,
}


impl<T> Default for PageParams<T> {
  fn default() -> Self {
    Self {
      max: None,
      draw_point: false,
      edit: false,
      style: Style::default(),
      styles: vec![],
      source: vec![],
    }
  }
}


impl<T: std::fmt::Display> PageParams<T> {
  pub fn init() -> Self {
    Self::default()
  }


  pub fn edit(mut self, b: bool) -> Self {
    self.edit = b; 
    self 
  }
  

  pub fn max(mut self, u: u16) -> Self {
    self.max = Some(u); 
    self
  }


  pub fn style(mut self, style: impl Into<Style>) -> Self {
    self.style = style.into();
    self
  }


  pub fn text(mut self, text: Vec<T>) -> Self {
    self.source = text;
    self.styles = self.source
      .iter().map(|t| TextParams::default()).collect();
    self
  }


  pub fn text_styles(
    mut self, text: Vec<T>, get_style: impl Fn(&T) -> TextParams,
  ) -> Self 
  {
    self.styles = text.iter().map(|t| get_style(t)).collect();
    self.source = text;
    self
  }


  pub fn draw_point(mut self, b: bool) -> Self {
    self.draw_point = b;
    self
  }
}


impl<T: std::fmt::Display> BuildView<Page<T>> for PageParams<T> {
  fn build(self, rect: &Rect) -> Page<T> {
    let (indexes, matrix) = print(
      rect.width().into(), 
      &self.styles, 
      &self.source
    );
    let matrix = if self.edit {
      PointMatrix::from(matrix).editor()
    } else {
      PointMatrix::from(matrix)
    };

    let mut point_view = PointView::from(rect);
    point_view.update(&matrix.point);

    Page {
      draw_point: self.draw_point,
      style: self.style,
      edit: self.edit,
      styles: self.styles,
      source: self.source,
      max: self.max,
      point_view,
      indexes, 
      matrix,
    }
  }
}


pub struct Page<T> {
  pub max: Option<u16>,
  pub draw_point: bool,
  pub style: Style,
  pub edit: bool,
  pub point_view: PointView,
  pub styles: Vec<TextParams>,
  pub source: Vec<T>,
  pub matrix: PointMatrix<char>, 
  pub indexes: Vec<usize>,
}


impl<T> Page<T> {
  pub fn get_index(&self) -> usize {
    self.indexes.get(self.matrix.point.y.head)
      .map(|u| u.clone())
      .unwrap_or(usize::MIN)
  }


  pub fn get_source(&self) -> Option<&T> {
    self.source.get(self.get_index())
  }


  pub fn get_string(&self) -> Option<String> {
    self.matrix.matrix.get(self.matrix.point.y.head)
      .map(|c| c.iter().collect())
  }


  pub fn delete(&mut self) {
    self.matrix.delete();
    self.point_view.update(&self.matrix.point);
  }


  pub fn backspace(&mut self) {
    self.matrix.backspace();
    self.point_view.update(&self.matrix.point);
  }


  pub fn insert(&mut self, ch: char) {
    self.matrix.insert(ch);
    self.point_view.update(&self.matrix.point);
  }


  pub fn move_left(&mut self, delta: usize) {
    self.matrix.move_left(delta);
    self.point_view.update(&self.matrix.point);
  }


  pub fn move_right(&mut self, delta: usize) {
    self.matrix.move_right(delta);
    self.point_view.update(&self.matrix.point);
  }


  pub fn move_down(&mut self, delta: usize) {
    self.matrix.move_down(delta);
    self.point_view.update(&self.matrix.point);
  }


  pub fn move_up(&mut self, delta: usize) {
    self.matrix.move_up(delta);
    self.point_view.update(&self.matrix.point);
  }


  pub fn get_view(&self, axis: CursorView) -> Vec<(&usize, &Vec<char>)> {
    self.indexes
      .iter()
      .zip(self.matrix.matrix.iter())
      .skip(axis.scroll)
      .take(usize::from(axis.size))
      .collect()
  }
}


impl<T: std::fmt::Display> Page<T> {
  pub fn get_param_string(&self) -> String {
    self.source
      .get(self.get_index())
      .map(|s| s.to_string())
      .unwrap_or("".to_string())
  }


  pub fn rebuild(
    &mut self,
    rect: &Rect,
  ) {
    let linear_head = self.matrix.get_linear_head();
    let (indexes, matrix) = print(
      rect.width().into(), 
      &self.styles, 
      &self.source
    );

    let matrix = if self.edit {
      PointMatrix::from(matrix).editor()
    } else {
      PointMatrix::from(matrix)
    };

    self.indexes = indexes;
    self.matrix = matrix;
    self.matrix.set_linear_head(linear_head);
    self.point_view.resize(&self.matrix.point, &rect);
  }


  pub fn restyle(
    &mut self,
    get_style: impl Fn(&T) -> TextParams,
  ) {
    self.styles = self.source.iter().map(|s| get_style(s)).collect();
    self.rebuild(&Rect::from(&self.point_view));
  }
}


impl<T: std::fmt::Display> Resize for Page<T> {
  fn resize(&mut self, rect: &Rect) {
    if self.point_view.get_width() == rect.width() {
      self.point_view.resize(&self.matrix.point, rect);
    } else {
      self.rebuild(rect);
    }
  }
}


impl<T: std::fmt::Display> GetHeight for Page<T> {
  fn get_height(&self) -> u16 {
    self.matrix.matrix.get_height().min(self.max.unwrap_or(u16::MAX))
  }
}


impl<T: std::fmt::Display> Draw for Page<T> {
  fn draw(&self, writer: &mut impl std::io::Write) 
    -> std::io::Result<()> 
  {
    use crossterm::{QueueableCommand, cursor, style};
    use unicode_width::UnicodeWidthChar;

    let (mut x, mut y) = self.point_view.pos().into();

    writer
      .queue(cursor::MoveTo(x, y))?
      .queue(style::SetAttribute(style::Attribute::Reset))?
      .queue(&self.style)?;

    for (index, line) in self.get_view(self.point_view.get_y_view()) {
      writer.queue(
        self.styles
          .get(*index)
          .map(|t| Style::from(t))
          .unwrap_or_default()
      )?;

      for (w, c) in self.point_view
        .get_x_view()
        .get_weighted_view(line) 
      {
        writer.queue(style::Print(c))?;
        x += w;
      }

      writer
        .queue(style::SetAttribute(style::Attribute::Reset))?
        .queue(&self.style)?;

      for _ in x..self.point_view.get_rect().x_end() {
        writer.queue(style::Print(' '))?;
      }

      x = self.point_view.pos().x(); 
      y += 1; 
      writer.queue(cursor::MoveTo(x, y))?;
    }

    writer.queue(style::SetAttribute(style::Attribute::Reset))?;

//    if self.draw_point {
//      self.point_view.draw(writer)?;
//    }

    Ok(())
  }
}
