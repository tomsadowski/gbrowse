// src/main.rs

#![allow(dead_code)]
//#![allow(unused_imports)]
#![allow(unused)]
#![allow(unused_mut)]
#![allow(unused_variables)]

mod app;
mod cursor;
mod dialog;
mod network;
mod gemdoc;
mod view;
mod tab;
mod text;
mod user;
mod keys;
mod style;
mod widget;
mod util;

use crossterm::QueueableCommand;
use std::io::Write;


fn main() -> std::io::Result<()> {
  // initialize app
  let mut app = {
    let args = std::env::args().collect::<Vec<String>>();
    let init = match args.get(1) {
      None       => user::INIT_FILE.into(),
      Some(init) => user::get_init_file(init),
    };
    let (w, h) = crossterm::terminal::size()?;
    app::App::init(&init, w, h)
  };
  let mut stdout = std::io::stdout();
  // register all keystrokes 
  crossterm::terminal::enable_raw_mode()?;
  // handle line wrapping manually
  stdout
    .queue(crossterm::terminal::EnterAlternateScreen)?
    .queue(crossterm::terminal::DisableLineWrap)?;
  // initial display
  app.write(&mut stdout)?;
  // break on control-c
  while !app.quit {
    if app.join_request() {
      app.write(&mut stdout)?;
    } 
    if crossterm::event::poll(std::time::Duration::from_millis(16))? {
      if let Some(message) = app.get_update(crossterm::event::read()?) {
        app.update(&message);
        app.write(&mut stdout)?;
      } 
    } 
  }
  // return terminal to normal state
  stdout
    .queue(crossterm::terminal::LeaveAlternateScreen)?
    .queue(crossterm::terminal::EnableLineWrap)?
    .queue(crossterm::cursor::SetCursorStyle::DefaultUserShape)?
    .flush()?;
  crossterm::terminal::disable_raw_mode()
}
