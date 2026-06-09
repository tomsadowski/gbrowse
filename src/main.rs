// src/main.rs

//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod app;
mod util;
mod frame;
pub use crate::frame::Frame;
mod textbox;
pub use crate::textbox::TextBox;
mod network;
pub use crate::network::Request;
mod screencursor;
pub use crate::screencursor::ScreenCursor;
mod userkeys;
pub use crate::userkeys::UserKeys;
mod userstyle;
pub use crate::userstyle::UserStyle;
mod user;
pub use crate::user::{
  User,
  UserTable,
};
mod tab;
pub use crate::tab::{
  Tab, 
  UrlTab, 
  TabManager,
};
mod cursor;
pub use crate::cursor::{
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
  CursorPlane,
};
mod dialog;
pub use crate::dialog::{
  Dialog, 
  DlgInput,
};
mod text;
pub use crate::text::{
  TextLine, 
  EditLine, 
  TextPlane,
};
mod gemini;
pub use crate::gemini::{
  GemTag, 
  GemText, 
  Status, 
  StatusText,
};
mod uiprimitives;
pub use crate::uiprimitives::{
  Action,
  Style, 
  TextStyle, 
  StyledText, 
  BorderStyle, 
  Margins, 
  ViewPort, 
  Rect, 
};


fn main() -> std::io::Result<()> {
  use crossterm::{
    QueueableCommand, terminal, event, cursor,
  };
  use std::{
    io::Write,
    time::Duration,
    env,
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
