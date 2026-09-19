// src/main.rs

//#![allow(unused_imports)]
#![allow(dead_code)]
//#![allow(unused_mut)]
//#![allow(unused_variables)]
//#![allow(unused_must_use)]

mod user;
mod userkeys;
mod userstyle;
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

pub use crate::dlg::{
    DialogParams,
    DialogType,
    Dialog,
};
pub use crate::userkeys::{
    KeyConfig,
};
pub use crate::userstyle::{
    StyleConfig,
};
pub use crate::network::{
    Request,
};
pub use crate::action::{
    Action,
};
pub use crate::view::{
    AppView,
    Tab, 
    TabText,
};
pub use crate::color::{
    Style, 
};
pub use crate::page::{
    TextParams, 
    Page,
    PageParams,
};
pub use crate::frame::{
    Frame,
    FrameParams,
    BorderParams, 
    MarginParams, 
};
pub use crate::user::{
    UserConfig,
    UserAssign,
    UserTable,
    AssignResult,
    AssignErr,
    ValueResult,
    ValueErr,
};
pub use crate::cursor::{
    Cursor, 
    PointMatrix,
    CursorVec,
    Point,
    PointView,
    CursorView,
};
pub use crate::layout::{
    Resize,
    Draw,
    BuildView,
    GetMaxHeight,
    GetDisplayHeight,
    ViewType,
    ViewTypeMut,
    resize_views,
    fill,
    build_opt_views,
};
pub use crate::rect::{
    Pos, 
    Dim,
    Rect, 
};
pub use crate::gemini::{
    GemTag, 
    GemText, 
    Status, 
    StatusText,
};


fn main() -> std::io::Result<()> {
    use crossterm::{QueueableCommand, terminal, event, cursor};
    use std::io::Write;

    // initialize app
    let mut app = {
        let args = std::env::args().collect::<Vec<String>>();
        let init = match args.get(1) {
            None => util::INIT_FILE.into(),
            Some(init) => util::get_init_file(init),
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
