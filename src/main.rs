// src/main.rs

//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod user;
mod userkeys;
mod userstyle;
mod cursortraits;
mod usertraits;
mod screencursor;
mod textcursor;
mod tab;
mod coreui;
mod frame;
mod styledtext;
mod textbox;
mod dialog;
mod network;
mod gemini;
mod app;
mod util;

pub use crate::user::User;
pub use crate::userkeys::UserKeys;
pub use crate::userstyle::UserStyle;
pub use crate::frame::Frame;
pub use crate::styledtext::StyledText;
pub use crate::textbox::TextBox;
pub use crate::network::Request;
pub use crate::screencursor::{
  ScreenCursor,
  LineCursor,
};
pub use crate::usertraits::{
  Assign,
  UserTable,
  UserFromStr,
};
pub use crate::cursortraits::{
  UnitCursor, 
  UnitCursorMut, 
  WeightedCursor, 
  CursorPlane,
};
pub use crate::textcursor::{
  TextLine, 
  EditLine, 
  TextPlane,
};
pub use crate::tab::{
  Tab, 
  UrlTab, 
  TabManager,
};
pub use crate::coreui::{
  Action,
  Style, 
  TextStyle, 
  BorderStyle, 
  Margins, 
  ViewPort, 
  ViewAxis,
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
