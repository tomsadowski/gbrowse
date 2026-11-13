// src/main.rs

#![allow(unused_imports)]
#![allow(dead_code)]
//#![allow(unused_mut)]
#![allow(unused_variables)]
//#![allow(unused_must_use)]

mod cfg;
mod cfgkeys;
mod cfgstyle;
mod cursor;
mod view;
mod dlg;
mod color;
mod rect;
mod frame;
mod page;
mod network;
mod gemini;
mod app;
mod action;
mod layout;
mod util;

pub use crate::{
    cfgkeys::{KeyConfig},
    cfgstyle::{StyleConfig},
    network::{Request},
    action::{Action},
    color::{Style},
    dlg::{DialogParams, DialogType, Dialog},
    view::{AppView, Tab, TabText},
    page::{TextParams, Page, PageParams},
    layout::{BuildView, View, resize_views, fill},
    rect::{Pos, Dim, Rect},
    frame::{Frame, FrameParams, BorderParams, MarginParams},
    gemini::{GemTag, GemText, Status, StatusText},
    cursor::{
        Cursor, 
        PointMatrix,
        CursorVec,
        Point,
        PointView,
        CursorView,
    },
    cfg::{
        UserConfig,
        UserAssign,
        UserTable,
        AssignResult,
        AssignErr,
        ValueResult,
        ValueErr,
    },
};


fn main() -> std::io::Result<()> {
    use crossterm::{QueueableCommand, terminal, event, cursor};
    use std::io::Write;
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .expect("Failed to open debug.log");
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format_timestamp_millis()
        .init();
    // initialize app
    let mut app = {
        let args = std::env::args().collect::<Vec<String>>();
        let init = match args.get(1) {
            None => util::INIT_FILE.into(),
            Some(init) => util::get_init_file(init),
        };
        let (w, h) = terminal::size()?;
        app::App::init(init, w, h)
    };
    let mut stdout = std::io::stdout();
    // register all keystrokes 
    terminal::enable_raw_mode()?;
    // handle line wrapping manually
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?;
    // initial display
    app.draw(&mut stdout)?;
    // break on control-c
    while !app.quit {
        if app.join_request() {
            app.draw(&mut stdout)?;
        } 
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Some(message) = app.get_update(event::read()?) {
                app.update(&message);
                app.draw(&mut stdout)?;
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
