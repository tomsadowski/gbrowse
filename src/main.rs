// src/main.rs

//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod user_traits;
pub use crate::user_traits::{
  Assign,
  UserTable,
  UserFromStr,
};
mod user;
pub use crate::user::User;
mod user_keys;
pub use crate::user_keys::UserKeys;
mod user_style;
pub use crate::user_style::UserStyle;

mod cursor_traits;
pub use crate::cursor_traits::{
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
  CursorPlane,
};
mod screen_cursor;
pub use crate::screen_cursor::ScreenCursor;
mod text_cursor;
pub use crate::text_cursor::{
  TextLine, 
  EditLine, 
  TextPlane,
};
mod tab;
pub use crate::tab::{
  Tab, 
  UrlTab, 
  TabManager,
};

mod ui_primitives;
pub use crate::ui_primitives::{
  Action,
  Style, 
  TextStyle, 
  BorderStyle, 
  Margins, 
  ViewPort, 
  Rect, 
};
mod frame;
pub use crate::frame::Frame;
mod styled_text;
pub use crate::styled_text::StyledText;
mod text_box;
pub use crate::text_box::TextBox;
mod dialog;
pub use crate::dialog::{
  Dialog, 
  DlgInput,
};

mod network;
pub use crate::network::Request;
mod gemini;
pub use crate::gemini::{
  GemTag, 
  GemText, 
  Status, 
  StatusText,
};

mod app;
mod util;


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
