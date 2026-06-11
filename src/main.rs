// src/main.rs

//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod user;
mod user_keys;
mod user_style;
mod cursor_traits;
mod user_traits;
mod screen_cursor;
mod text_cursor;
mod tab;
mod core_ui;
mod frame;
mod styled_text;
mod text_box;
mod dialog;
mod network;
mod gemini;
mod app;
mod util;

pub use crate::screen_cursor::ScreenCursor;
pub use crate::user::User;
pub use crate::user_keys::UserKeys;
pub use crate::user_style::UserStyle;
pub use crate::frame::Frame;
pub use crate::styled_text::StyledText;
pub use crate::text_box::TextBox;
pub use crate::network::Request;
pub use crate::user_traits::{
  Assign,
  UserTable,
  UserFromStr,
};
pub use crate::cursor_traits::{
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
  CursorPlane,
};
pub use crate::text_cursor::{
  TextLine, 
  EditLine, 
  TextPlane,
};
pub use crate::tab::{
  Tab, 
  UrlTab, 
  TabManager,
};
pub use crate::core_ui::{
  Action,
  Style, 
  TextStyle, 
  BorderStyle, 
  Margins, 
  ViewPort, 
  Rect, 
};
pub use crate::dialog::{
  Dialog, 
  DlgInput,
};
pub use crate::gemini::{
  GemTag, 
  GemText, 
  Status, 
  StatusText,
};


fn main() -> std::io::Result<()> {
  use crossterm::{
    QueueableCommand, terminal, event, cursor,
  };
  use std::{
    io::Write, time::Duration, env,
  };

  // initialize app
  let mut app = {
    let args = env::args().collect::<Vec<String>>();
    let init = match args.get(1) {
      None       => user::INIT_FILE.into(),
      Some(init) => user::get_init_file(init),
    };
    let (w, h) = terminal::size()?;
    app::App::init(&init, w, h)
  };
  let mut stdout = std::io::stdout();
  // register all keystrokes 
  terminal::enable_raw_mode()?;
  // handle line wrapping manually
  stdout
    .queue(terminal::EnterAlternateScreen)?
    .queue(terminal::DisableLineWrap)?;
  // initial display
  app.write(&mut stdout)?;
  // break on control-c
  while !app.quit {
    if app.join_request() {
      app.write(&mut stdout)?;
    } 
    if event::poll(Duration::from_millis(16))? {
      if let Some(message) = app.get_update(event::read()?) {
        app.update(&message);
        app.write(&mut stdout)?;
      } 
    } 
  }
  // return terminal to normal state
  stdout
    .queue(terminal::LeaveAlternateScreen)?
    .queue(terminal::EnableLineWrap)?
    .queue(cursor::SetCursorStyle::DefaultUserShape)?
    .flush()?;
  terminal::disable_raw_mode()
}
