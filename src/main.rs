// src/main.rs

#![allow(dead_code)]
//#![allow(unused_imports)]
#![allow(unused)]
#![allow(unused_mut)]
#![allow(unused_variables)]

mod app;
mod common;
mod cursor;
mod frame;
mod protocol;
mod rect;
mod tab;
mod text;
mod user;
mod keys;
mod style;
mod widget;

use crate::{
  common as c,
  app::App,
};
use crossterm::{
  QueueableCommand,
  terminal,
  event,
  cursor::{SetCursorStyle},
};
use std::{
  env,
  time::Duration,
  io::{self, Write, stdout},
};

fn main() -> io::Result<()> {
  // initialize app
  let mut app = {
    let args   = env::args().collect::<Vec<String>>();
    let (w, h) = terminal::size()?;
    let init = match args.get(1) {
      Some(init) => format!("{}/{}", c::USER_DATA, init),
      None       => format!("{}/{}", c::USER_DATA, c::USER_INIT),
    };
    App::init(&init, w, h)
  };
  let mut stdout = stdout();
  // register keystrokes 
  terminal::enable_raw_mode()?;
  // handle line wrapping manually
  stdout
    .queue(terminal::EnterAlternateScreen)?
    .queue(terminal::DisableLineWrap)?;
  // initial display
  app.write(&mut stdout)?;
  // break on control-c
  while !app.quit {
    if app.try_join_request() {
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
    .queue(SetCursorStyle::DefaultUserShape)?
    .flush()?;
  terminal::disable_raw_mode()
}
