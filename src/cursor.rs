// src/cursor.rs

use crate::{Rect, Pos, Dim};

pub struct PointMatrix<T> {
  pub point:  Point,
  pub matrix: Vec<Vec<T>>,
}

impl<T> Default for PointMatrix<T> {
  fn default() -> Self {
    Self {
      point: Point::default(),
      matrix: vec![],
    }
  }
}

impl<T> From<Vec<Vec<T>>> for PointMatrix<T> {
  fn from(matrix: Vec<Vec<T>>) -> Self {
    Self {
      point: Point::default(),
      matrix,
    }
  }
}

impl<T> PointMatrix<T> {
  pub fn init() -> Self { Self::default() }

  pub fn editor(mut self) -> Self {
    self.make_editor();
    self
  }

  pub fn make_editor(&mut self) {
    self.point.make_editor(&self.matrix)
  }

  pub fn get_linear_head(&self) -> usize {
    self.point.get_linear_head(&self.matrix)
  }

  pub fn set_linear_head(&mut self, idx: usize) {
    self.point.set_linear_head(&self.matrix, idx)
  }

  pub fn move_y(&mut self, idelta: isize) -> bool {
    self.point.move_y(&self.matrix, idelta)
  }

  pub fn move_x(&mut self, idelta: isize) -> isize {
    self.point.move_x(&self.matrix, idelta)
  }

  pub fn delete(&mut self) -> bool {
    self.point.delete(&mut self.matrix)
  }

  pub fn backspace(&mut self) -> bool {
    self.point.backspace(&mut self.matrix)
  }

  pub fn insert(&mut self, t: T) -> bool {
    self.point.insert(&mut self.matrix, t)
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
}

#[derive(Default)]
pub struct CursorVec<T> {
  pub cursor: Cursor,
  pub vec:    Vec<T>,
}

impl<T> From<T> for CursorVec<T> {
  fn from(t: T) -> Self {
    Self {
      cursor: Cursor::default(),
      vec: vec![t],
    }
  }
}

impl<T> From<Vec<T>> for CursorVec<T> {
  fn from(vec: Vec<T>) -> Self {
    Self {
      cursor: Cursor::default(),
      vec,
    }
  }
}

impl<T> std::ops::Deref for CursorVec<T> {
  type Target = Vec<T>;
  fn deref(&self) -> &Self::Target {&self.vec}
}

impl<T> std::ops::DerefMut for CursorVec<T> {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.vec }
}

impl<T> CursorVec<T> {
  pub fn move_head(&mut self, mut idelta: isize) -> isize {
    self.cursor.move_head(&self.vec, idelta)
  }

  pub fn move_wrapped(&mut self, mut idelta: isize) -> MoveCommand {
    self.cursor.move_wrapped(&self.vec, idelta)
  }

  pub fn get_current(&self) -> Option<&T> {
    self.vec.get(*self.cursor)
  }

  pub fn get_current_mut(&mut self) -> Option<&mut T> {
    self.vec.get_mut(*self.cursor)
  }

  pub fn remove(&mut self) -> Option<RemoveCommand> {
    self.cursor.remove(&mut self.vec)
  }

  pub fn delete(&mut self) -> Option<RemoveCommand> {
    self.cursor.delete(&mut self.vec)
  }

  pub fn backspace(&mut self) -> Option<RemoveCommand> {
    self.cursor.backspace(&mut self.vec)
  }

  pub fn insert(&mut self, t: T) -> InsertCommand {
    self.cursor.insert(&mut self.vec, t)
  }

  pub fn apply_command(&mut self, cmd: CursorCommand<T>) {
    cmd.apply_to_vec(self)
  }

  pub fn insert_unique_with(
    &mut self, 
    is_equal: impl Fn(&T) -> bool, 
    unit:     T
  ) -> Option<InsertCommand> {
    self.cursor.insert_unique_with(&mut self.vec, is_equal, unit)
  }
}

pub enum CursorCommand<T> {
  Move(MoveCommand),
  Remove(RemoveCommand),
  Insert(InsertCommand, T),
}

impl<T> From<MoveCommand> for CursorCommand<T> {
  fn from(cmd: MoveCommand) -> Self {
    Self::Move(cmd)
  }
}

impl<T> From<RemoveCommand> for CursorCommand<T> {
  fn from(cmd: RemoveCommand) -> Self {
    Self::Remove(cmd)
  }
}

impl<T> From<(InsertCommand, T)> for CursorCommand<T> {
  fn from((cmd, t): (InsertCommand, T)) -> Self {
    Self::Insert(cmd, t)
  }
}

impl<T> CursorCommand<T> {
  pub fn apply_to_vec(self, vec: &mut CursorVec<T>) {
    match self {
      Self::Move(move_cmd)        => { move_cmd.apply_to_vec(vec); }
      Self::Remove(remove_cmd)    => { remove_cmd.apply_to_vec(vec); }
      Self::Insert(insert_cmd, t) => { insert_cmd.apply_to_vec(vec, t); }
    }
  }
}

pub enum MoveCommand {
  Set(usize),
  Move(isize), 
}

impl MoveCommand {
  pub fn apply<T>(&self, cursor: &mut Cursor, vec: &Vec<T>) -> isize {
    match self {
      Self::Set(u)     => {cursor.head = *u; 0},
      Self::Move(i)    => cursor.move_head(vec, *i),
    }
  }

  pub fn apply_to_vec<T>(&self, vec: &mut CursorVec<T>) -> isize {
    match self {
      Self::Set(u)     => {vec.cursor.head = *u; 0},
      Self::Move(i)    => vec.cursor.move_head(&vec.vec, *i),
    }
  }
}

pub struct RemoveCommand(usize, MoveCommand);

impl RemoveCommand {
  pub fn apply<T>(&self, vec: &mut Vec<T>) -> &MoveCommand {
    vec.remove(self.0); &self.1
  }

  pub fn apply_to_vec<T>(&self, vec: &mut CursorVec<T>) {
    self.apply(&mut vec.vec).apply_to_vec(vec);
  }
}

pub enum InsertCommand { 
  Insert(usize, MoveCommand), 
  Push(MoveCommand),
}

impl InsertCommand {
  pub fn apply<T>(&self, vec: &mut Vec<T>, t: T) -> &MoveCommand {
    match self {
      Self::Insert(u, mov) => { vec.insert(*u, t); mov }
      Self::Push(mov)      => { vec.push(t);       mov }
    }
  }

  pub fn apply_to_vec<T>(&self, vec: &mut CursorVec<T>, t: T) {
    self.apply(&mut vec.vec, t).apply_to_vec(vec);
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Cursor {
  head: usize,
  buff: bool,
}

impl std::ops::Deref for Cursor {
  type Target = usize;
  fn deref(&self) -> &Self::Target {&self.head}
}

impl std::ops::DerefMut for Cursor {
  fn deref_mut(&mut self) -> &mut Self::Target {&mut self.head}
}

impl Cursor {
  pub fn editor<T>(mut self, vec: &Vec<T>) -> Self {
    self.make_editor(vec);
    self
  }

  pub fn make_editor<T>(&mut self, vec: &Vec<T>) {
    self.buff = true;
    self.move_to_end(vec);
  }

  pub fn get_max<T>(&self, vec: &Vec<T>) -> usize {
    if self.buff { vec.len() } 
    else         { vec.len().saturating_sub(1) }
  }

  pub fn peek_move<T>(&self, vec: &Vec<T>, idelta: isize) -> isize {
    self.clone().move_head(vec, idelta)
  }

  pub fn fit<T>(&mut self, vec: &Vec<T>, new_head: usize) {
    self.head = self.get_max(vec).min(new_head);
  }

  pub fn move_to_start(&mut self) {
    self.head = 0;
  }

  pub fn move_to_end<T>(&mut self, vec: &Vec<T>) {
    self.head = self.get_max(vec);
  }

  pub fn move_wrapped<T>(&mut self, vec: &Vec<T>, mut idelta: isize) 
  -> MoveCommand 
  {
    let     imax       = self.get_max(vec) as isize;
    let mut iremainder = self.move_head(vec, idelta);
    if iremainder == 0 {
      MoveCommand::Set(self.head)
    } else {
      self.move_head(vec, 
        vec.len() as isize * iremainder.signum() * -1
      );
      self.move_wrapped(vec, 
        (iremainder.saturating_abs() - 1) * iremainder.signum()
      )
    }
  }

  pub fn move_head<T>(&mut self, vec: &Vec<T>, mut idelta: isize) -> isize {
    let ihead = self.head as isize;
    let imax = self.get_max(vec) as isize;
    let new_ihead = ihead + idelta;
    if new_ihead < 0 {
      self.head = 0;
      new_ihead
    } else if new_ihead > imax {
      self.head = self.get_max(vec);
      new_ihead - imax
    } else {
      self.head = new_ihead as usize;
      0
    }
  }

  pub fn remove<T>(&mut self, vec: &mut Vec<T>) -> Option<RemoveCommand> {
    if self.head >= vec.len() {None}
    else {
      vec.remove(self.head);
      let head = self.head;
      self.move_wrapped(vec, -1);
      Some(RemoveCommand(head, MoveCommand::Set(self.head)))
    }
  }

  pub fn delete<T>(&self, vec: &mut Vec<T>) -> Option<RemoveCommand> {
    if self.head >= vec.len() {None}
    else {
      vec.remove(self.head);
      Some(RemoveCommand(self.head, MoveCommand::Move(0)))
    } 
  }

  pub fn backspace<T>(&mut self, vec: &mut Vec<T>) -> Option<RemoveCommand> {
    if self.peek_move(vec, -1) != 0 {None}
    else {
      self.move_head(vec, -1);
      vec.remove(self.head);
      Some(RemoveCommand(self.head, MoveCommand::Move(-1)))
    } 
  }

  pub fn insert_unique_with<T>(
    &mut self, 
    vec:        &mut Vec<T>, 
    is_equal:   impl Fn(&T) -> bool, 
    unit:       T
  ) -> Option<InsertCommand> {
    if let Some((idx, _)) = vec
      .iter_mut()
      .enumerate()
      .find(|(_, u)| is_equal(u))
    {
      self.head = idx;
      None
    } else if vec.len() == 0 {
      vec.push(unit);
      Some(InsertCommand::Push(MoveCommand::Move(0)))
    } else if self.head + 1 == vec.len() {
      vec.push(unit);
      self.head += 1;
      Some(InsertCommand::Push(MoveCommand::Move(1)))
    }
    else {
      self.head += 1;
      vec.insert(self.head, unit);
      Some(InsertCommand::Insert(self.head, MoveCommand::Move(1)))
    }
  }

  pub fn insert<T>(&mut self, vec: &mut Vec<T>, c: T) -> InsertCommand {
    if self.head + 1 == vec.len() || vec.len() == 0 {
      vec.push(c);
      self.move_head(vec, 1);
      InsertCommand::Push(MoveCommand::Move(1))
    } else {
      let head = self.head;
      vec.insert(self.head, c);
      self.move_head(vec, 1);
      InsertCommand::Insert(head, MoveCommand::Move(1))
    }
  }

  pub fn get_weighted_head(&self, vec: Vec<char>) -> usize {
    use unicode_width::UnicodeWidthChar;
    vec
      .iter()
      .take(self.head)
      .map(|c| c.width().unwrap_or(0))
      .sum()
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
  pub x:  Cursor,
  pub y:  Cursor,
  pref_x: usize,
}

impl Point {
  pub fn init() -> Self { Self::default() }

  pub fn editor<T>(mut self, vec: &Vec<Vec<T>>) -> Self {
    self.make_editor(vec);
    self
  }

  pub fn make_editor<T>(&mut self, vec: &Vec<Vec<T>>) {
    vec.get(*self.y).map(|v| self.x.make_editor(v));
  }

  pub fn get_linear_head<T>(&self, vec: &Vec<Vec<T>>) -> usize {
    vec
      .iter()
      .take(self.y.saturating_sub(1))
      .map(|v| v.len().max(1))
      .chain(std::iter::once(*self.x))
      .sum()
  }

  pub fn set_linear_head<T>(&mut self, vec: &Vec<Vec<T>>, idx: usize) {
    self.y.move_to_start();
    self.x.move_to_start();
    self.move_x(vec, idx as isize);
  }

  pub fn move_y<T>(&mut self, vec: &Vec<Vec<T>>, idelta: isize) -> bool {
    if self.y.move_head(vec, idelta) != idelta {
      vec.get(*self.y).map(|v| self.x.fit(v, self.pref_x));
      true
    } else {false}
  }

  pub fn move_x<T>(&mut self, vec: &Vec<Vec<T>>, idelta: isize) -> isize {
    let iremainder = vec
      .get(*self.y)
      .map(|v| self.x.move_head(v, idelta))
      .unwrap_or(0);
    if iremainder != 0 && self.y.move_head(vec, iremainder.signum()) == 0 {
      match vec.get(*self.y) {
        None => iremainder,
        Some(v) => {
          self.x.move_head(v, v.len() as isize * iremainder.signum() * -1);
          self.move_x(vec, 
            (iremainder.saturating_abs() - 1) * iremainder.signum()
          )
        }
      }
    } else {
      self.pref_x = self.x.head;
      iremainder
    }
  }

  pub fn delete<T>(&mut self, vec: &mut Vec<Vec<T>>) -> bool {
    vec
      .get_mut(*self.y)
      .map(|c| self.x.delete(c))
      .is_some()
  }

  pub fn backspace<T>(&mut self, vec: &mut Vec<Vec<T>>) -> bool {
    vec
      .get_mut(*self.y)
      .map(|c| self.x.backspace(c))
      .is_some()
  }

  pub fn insert<T>(&mut self, vec: &mut Vec<Vec<T>>, t: T) -> bool {
    vec
      .get_mut(*self.y)
      .map(|c| self.x.insert(c, t))
      .is_some()
  }

  pub fn move_left<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_x(vec, delta as isize * -1) == 0
  }

  pub fn move_right<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_x(vec, delta as isize) == 0
  }

  pub fn move_down<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_y(vec, delta as isize)
  }

  pub fn move_up<T>(&mut self, vec: &Vec<Vec<T>>, delta: usize) -> bool {
    self.move_y(vec, delta as isize * -1)
  }
}

pub fn get_weighted_length(vec: &Vec<char>) -> usize {
  use unicode_width::UnicodeWidthChar;
  vec.iter().map(|c| c.width().unwrap_or(0)).sum()
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PointView {
  pos: Pos,
  x: CursorView,
  y: CursorView,
}

impl From<&Rect> for PointView {
  fn from(rect: &Rect) -> Self {
    let rect = rect.clone();
    Self {
      x: CursorView::from_size(rect.width()),
      y: CursorView::from_size(rect.height()),
      pos: rect.pos(),
    }
  }
}

impl PointView {
  pub fn get_x_view(&self)   -> CursorView {self.x}
  pub fn get_y_view(&self)   -> CursorView {self.y}
  pub fn dim(&self)          -> Dim { Dim(self.x.size, self.y.size) }  
  pub fn pos(&self)          -> Pos { self.pos }
  pub fn get_x_cursor(&self) -> u16 { self.pos.x() + self.x.cursor }
  pub fn get_y_cursor(&self) -> u16 { self.pos.y() + self.y.cursor }
  pub fn get_width(&self)    -> u16   {self.x.size}
  pub fn get_height(&self)   -> u16   {self.y.size}
  pub fn get_x_scroll(&self) -> usize {self.x.scroll}
  pub fn get_y_scroll(&self) -> usize {self.y.scroll}
  pub fn get_rect(&self) -> Rect { 
    Rect::from(self.dim()).with_pos(self.pos()) 
  }

  pub fn resize(&mut self, point: &Point, rect: &Rect) {
    let rect = rect.clone();
    self.pos = rect.pos();
    self.y.resize(*point.y, rect.height());
    self.x.resize(*point.x, rect.width());
    //eprintln!("{:#?}", self);
  }

  pub fn update(&mut self, point: &Point) -> bool {
    //eprintln!("{:#?}", self);
    let y = self.y.update(*point.y);
    let x = self.x.update(*point.x);
    x || y
  }

  pub fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor};
    w
      .queue(cursor::MoveTo(self.get_x_cursor(), self.get_y_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CursorView {
  pub head:   usize, // data head
  pub scroll: usize, // start of displayable data
  pub cursor: u16,   // on-screen cursor
  pub size:   u16,   // width or height of rectangle
}

impl CursorView {
  pub fn from_size(size: u16) -> Self {
    Self {
      scroll: 0, 
      head:   0, 
      cursor: 0, 
      size,
    }
  }

  pub fn get_unit_view<T>(self, vec: &Vec<T>) -> Vec<&T> {
    vec
      .iter()
      .skip(self.scroll)
      .take(self.size.into())
      .collect() 
  }

  pub fn get_weighted_view(self, vec: &[char]) -> Vec<(u16, &char)> {
    use unicode_width::UnicodeWidthChar;
    let size         = usize::from(self.size);  
    let mut text     = vec.iter().skip(self.scroll);
    let mut acc_size = 0;
    let mut result   = vec![];
    while let Some(c) = text.next() && acc_size < size {
      let width = c.width().unwrap_or(0);
      acc_size += width;
      result.push((u16::try_from(width).unwrap_or(u16::MIN), c));
    }
    result
  }

  // preserve cursor position if it still fits in the new bounds
  pub fn resize(
    &mut self, 
    new_head:  usize, 
    new_size:  u16
  ) {
    // position of cursor on screen
    self.size  = new_size;
    self.head  = new_head;
    // go to beginning of line
    if new_head < usize::from(new_size) {
      self.scroll = 0;
      self.cursor = u16::try_from(self.head).unwrap();
    // position must be lowered to fit within new bounds
    } else if self.cursor > new_size - 1 {
      self.cursor = self.size - 1;
      self.scroll = self.head - usize::from(self.size - 1);
    // position can be preserved
    } else {
      self.scroll = self.head.saturating_sub(self.cursor.into());
    }
  }

  pub fn update(&mut self, new_head: usize) -> bool {
    // no move
    if self.head == new_head {
      false
    // move forward
    } else if self.head < new_head {
      let delta_size = new_head - self.head;
      let max_delta  = self.size
        .saturating_sub(1)
        .saturating_sub(self.cursor);
      // no scroll
      if delta_size < usize::from(max_delta) { 
        self.cursor  += u16::try_from(delta_size).unwrap();
        self.head     = new_head;
        false
      // scroll forward
      } else {
        self.scroll += delta_size - usize::from(max_delta);
        self.cursor += max_delta;
        self.head    = new_head;
        true
      }
    // move backward
    } else { 
      let delta_size = self.head - new_head;
      // no scroll
      if delta_size <= usize::from(self.cursor) {
        self.cursor -= u16::try_from(delta_size).unwrap();
        self.head    = new_head;
        false
      // scroll backward
      } else { 
        self.scroll = self.scroll.saturating_sub(
          delta_size - usize::from(self.cursor)
        );
        self.cursor = u16::try_from(new_head - self.scroll).unwrap();
        self.head = new_head;
        true
      }
    } 
  }
}
