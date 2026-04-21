// src/main.rs

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused)]
#![allow(unused_mut)]
#![allow(unused_variables)]

mod text;
mod widget;
mod user;
mod common;
mod protocol;
mod dialog;
mod tab;
mod message;

use crate::{
  common as c,
  message::{Message, Task, Action, Focus},
  user::{User},
  tab::{Tab, TabList},
  dialog::{Response, Dialog},
  text::{StyledText, Linear, Style}, 
  widget::{Rect, Frame, TextBox, EditBox, cursor_hide, PlaneWidget},
  protocol::{Request, GemDoc, GemTag, Status, Scheme},
};
use crossterm::{
  QueueableCommand,
  cursor::{self, SetCursorStyle},
  terminal::{self, Clear, ClearType},
  event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use url::Url;
use std::{
  fs, thread, env,
  sync::mpsc,
  time::Duration,
  str::FromStr,
  io::{self, Write, Read, stdout, Stdout},
};

fn main() -> io::Result<()> {
  let args         = env::args().collect::<Vec<String>>();
  let default_path = String::from(c::START);
  let init_path    = args.get(1).unwrap_or(&default_path); 
  let mut stdout   = stdout();
  // register keystrokes 
  terminal::enable_raw_mode()?;
  // handle line wrapping manually
  stdout
    .queue(terminal::EnterAlternateScreen)?
    .queue(terminal::DisableLineWrap)?;
  // initialize app
  let mut app = {
    let (w, h) = terminal::size()?;
    App::init(init_path, w, h)
  };
  let mut request_maybe = app.try_request(&app.user.init_url.clone());
  // first message is Request, which will request the initial url
  let mut quit = false;
  let mut write = true;
  // initial display
  app.write(&mut stdout)?;
  // break on control-c
  while !quit {
    write = false;
    if let Some(request) = &mut request_maybe {
      if request.handle.is_finished() {
        let result = request.rx.recv().unwrap()
          .map(|(r, c)| GemDoc::new(&request.url, r, c)).flatten();
        match result {
          Err(e)     => app.ack(Task::Default, &e),
          Ok(gemdoc) => app.set_gemdoc(&request.url, gemdoc),
        }
        request_maybe = None;
        app.pending = false;
        app.write(&mut stdout)?;
      }
    } 
    if event::poll(Duration::from_millis(16))? {
      if let Some(message) = app.get_update(event::read()?) {
        if let Some(request) = app.update(&message) {
          if let None = &mut request_maybe {
            request_maybe = Some(request);
            app.pending = true;
          }
        } else if let Message::Quit = message {
          quit = true;
        }
        write = true;
      } 
    } 
    if write {
      app.write(&mut stdout)?;
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

pub struct App {
  pub init_path: String,
  pub frame:     Frame,
  pub user:      User,
  pub urls:      Vec<String>,
  pub rect:      Rect,
  pub tabs:      TabList,
  pub focus:     Focus,
  pub new_dlg:   bool,
  pub pending:   bool,
  pub clear:     bool,
} 
impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {
    let user_text = fs::read_to_string(path).unwrap_or("".into());
    let user      = User::from_str(&user_text).unwrap_or_default();
    let frame     = user.get_frame(&Rect::new(w, h));
    let rect      = frame.inner_rect.clone();
    let tab       = Tab::init(&rect, &user.init_url);
    let urls: Vec<String> = 
      match fs::read_to_string(&user.save_file) {
        Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
        Err(e) => vec![],
      };
    Self {
      frame,
      user,
      rect,
      urls,
      init_path: path.into(),
      tabs:      TabList::new(tab),
      focus:     Focus::Tab,
      new_dlg:   false,
      pending:   false,
      clear:     true
    }
  }
  fn ack(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("Press {} to acknowledge", self.user.keys.ack);
    let dialog   = Dialog::ack(prompt, help, style, &self.rect);
    self.focus   = Focus::Dialog(task, dialog);
    self.new_dlg = true;
  }
  fn ask(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("{} yes {} no", self.user.keys.yes, self.user.keys.no);
    let dialog   = Dialog::ask(prompt, help, style, &self.rect);
    self.focus   = Focus::Dialog(task, dialog);
    self.new_dlg = true;
  }
  fn text(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let dialog   = Dialog::text(prompt, style, &self.rect);
    self.focus   = Focus::Dialog(task, dialog);
    self.new_dlg = true;
  }
  fn select_url(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let dialog   = Dialog::select(prompt, self.urls.clone(), style, &self.rect);
    self.focus   = Focus::Dialog(task, dialog);
    self.new_dlg = true;
  }
  fn get_init_url(&mut self) -> Option<Url> {
    match Url::parse(&self.user.init_url) {
      Ok(url) => Some(url),
      Err(e) => {
        self.ack(Task::Default, &e.to_string());
        None
      }
    }
  }
  fn reload_config(&mut self, path: Option<&str>) {
    let path      = path.unwrap_or(&self.init_path);
    let user_text = fs::read_to_string(path).unwrap_or("".into());
    self.user     = User::from_str(&user_text).unwrap_or_default();
  }
  fn set_gemdoc(&mut self, url: &Url, gemdoc: GemDoc) {
    self.tabs.add(url.as_str());
    match gemdoc.status.tag {
      Status::InputExpected | Status::InputExpectedSensitive => {
        self.text(Task::Reply, &gemdoc.status.txt);
      }
      Status::RedirectTemporary | Status::RedirectPermanent => {
        self.tabs.url_str.push_str(&gemdoc.status.txt);
        self.ask(Task::Redirect(gemdoc.status.txt.clone()), &gemdoc.status.txt);
      }
      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {
        self.ack(Task::Default, &gemdoc.status.txt);
      }
      _ => {}
    };
    self.tabs.content = self.user.get_gem_textbox(&self.rect, &gemdoc);
    self.tabs.gemdoc  = Some(gemdoc);
  }
  fn try_request(&mut self, url_str: &str) -> Option<Request> {
    match Url::parse(url_str) {
      Err(e)  => {
        self.ack(Task::Default, &format!("URL parse error: {}", &e)); 
        None
      }
      Ok(url) => 
        Some(Request::new(&url, self.user.timeout)),
    }
  }
  fn update(&mut self, message: &Message) -> Option<Request> {
    self.clear = false;
    self.new_dlg = false;
    self.tabs.reset_state();
    match message {
      Message::Resize(w, h) => {
        self.frame.resize(&Rect::new(*w, *h));
        self.rect = self.frame.inner_rect.clone();
        if let Focus::Dialog(_, dialog) = &mut self.focus {
          dialog.resize(&self.rect);
        }
        self.tabs.resize(&self.rect);
        self.clear = true;
        None
      }
      Message::Action(action) => match &mut self.focus {
        Focus::Dialog(task, dialog) => match &mut dialog.response {
          Response::Ack(_) => {
            self.focus = Focus::Tab; None
          } 
          Response::Ask(_) => match action {
            Action::Yes => match task {
              Task::Go(url) => {
                let url = url.clone();
                self.focus = Focus::Tab;
                self.try_request(&url)
              }
              Task::Redirect(url_str) => {
                let text = url_str.trim().replace(" ", "%20");
                self.focus = Focus::Tab;
                self.try_request(&format!("{}?{}", self.tabs.url_str, text))
              }
              Task::DelTab => {
                self.tabs.delete();
                self.focus = Focus::Tab; None
              }
              _ => {
                self.focus = Focus::Tab; None
              }
            }
            Action::No | Action::Cancel => {
              self.focus = Focus::Tab; None
            }
            _ => None,
          } 
          Response::Select(textbox) => match action {
            Action::Inspect => match task {
              Task::NewTab => {
                if self.urls.len() > 0 {
                  let url_str = &self.urls[textbox.content.get_source_idx()].clone();
                  self.focus = Focus::Tab;
                  self.try_request(url_str)
                } else {
                  self.focus = Focus::Tab;
                  None
                }
              }
              _ => None,
            }
            Action::MoveLeft => {
              textbox.left(1); None
            }
            Action::MoveRight => {
              textbox.right(1); None
            }
            Action::MoveUp => {
              textbox.up(1); None
            }
            Action::MoveDown => {
              textbox.down(1); None
            }
            Action::Top => {
              textbox.up(textbox.y_len()); None
            }
            Action::Bottom => {
              textbox.down(textbox.y_len()); None
            }
            Action::PageUp => {
              textbox.up(usize::from(textbox.rect.h)); None
            } 
            Action::PageDown => {
              textbox.down(usize::from(textbox.rect.h)); None
            }
            Action::Cancel => {
              self.focus = Focus::Tab; None
            }
            _ => None,
          }
          Response::Text(editbox) => match action {
            Action::Enter => match task {
              Task::Reply => {
                let text = editbox.content.text.to_string();
                let text = text.trim().replace(" ", "%20");
                self.try_request(&format!("{}?{}", self.tabs.url_str, text))
              }
              Task::NewTab => {
                let text = editbox.content.text.to_string();
                self.try_request(&format!("{}?{}", self.tabs.url_str, text))
              }
              _ => None,
            }
            Action::MoveLeft => {
              editbox.left(1); None
            }
            Action::MoveRight => {
              editbox.right(1); None
            }
            Action::Delete => {
              editbox.delete(); None
            }
            Action::Backspace => {
              editbox.backspace(); None
            }
            Action::Insert(c) => {
              editbox.insert(*c); None
            }
            Action::Cancel => {
              self.focus = Focus::Tab; None
            }
            _ => None,
          }
        }
        Focus::Tab => match action {
          Action::MoveLeft => {
            self.tabs.left(1); None
          }
          Action::MoveRight => {
            self.tabs.right(1); None
          }
          Action::MoveUp => {
            self.tabs.up(1); None
          }
          Action::MoveDown => {
            self.tabs.down(1); None
          }
          Action::Top => {
            let len = self.tabs.y_len(); 
            self.tabs.up(len); None
          }
          Action::Bottom => {
            let len = self.tabs.y_len(); 
            self.tabs.down(len); None
          }
          Action::PageUp => {
            let len = self.tabs.rect.h; 
            self.tabs.up(usize::from(len)); None
          }
          Action::PageDown => {
            let len = self.tabs.rect.h; 
            self.tabs.down(usize::from(len)); None
          }
          Action::LoadUrl => {
            self.select_url(Task::NewTab, "choose the url: "); None
          }
          Action::SaveUrl => {
            let url_str = self.tabs.url_str.clone();
            // only add url_str if new
            if !self.urls.iter().any(|url| **url == url_str) {
              self.urls.push(url_str);
              // write to save file
              match fs::OpenOptions::new().write(true).truncate(true).open(&self.user.save_file) {
                Err(e) => {
                  self.ack(Task::Default, &format!("could not create save file: {}", &e)); 
                  None
                }
                Ok(mut f) => {
                  for url in self.urls.iter() {
                    f.write(&format!("{}\n", url).as_bytes());
                  }
                  None
                }
              }
            } else {None}
          }
          Action::NewTab => {
            self.text(Task::NewTab, "enter path: "); None
          }
          Action::DelTab => {
            self.ask(Task::DelTab, "Delete current tab?");
            self.tabs.delete(); None
          }
          Action::CycleLeft => {
            if self.tabs.len() > 1 {
              self.tabs.wrapping_backward(1);
            }
            None
          }
          Action::CycleRight => {
            if self.tabs.len() > 1 {
              self.tabs.wrapping_forward(1);
            }
            None
          }
          Action::Inspect => {
            if let Some(gemdoc) = &self.tabs.gemdoc {
              match gemdoc.doc[self.tabs.get_source_idx()].tag.clone() {
                GemTag::Link(Scheme::Gemini, url) => {
                  let prompt = &format!("go to {}?", url);
                  self.ask(Task::Go(url.into()), prompt);
                }
                GemTag::Link(_, url) => {
                  self.ack(Task::Default, &format!("Protocol {} not yet supported", url));
                }
                gemtext => {
                  self.ack(Task::Default, &format!("you've selected {:?}", gemtext));
                }
              }
              None
            } else {None}
          }
          _ => None,
        }
      }
      _ => None,
    }
  }
  fn get_update(&self, event: Event) -> Option<Message> {
    match event {
      Event::Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c'), ..
      }) =>
        Some(Message::Quit),
      Event::Resize(w, h) => 
        Some(Message::Resize(w, h)),
      Event::Key(KeyEvent {
        code: kc, kind: KeyEventKind::Press, ..
      }) => match &self.focus {
        Focus::Dialog(task, dialog) => match &dialog.response {
          Response::Ack(_) => 
            self.user.keys.get_ack_dialog_action(&kc)
              .map(|a| Message::Action(a)),
          Response::Ask(_) => 
            self.user.keys.get_ask_dialog_action(&kc)
              .map(|a| Message::Action(a)),
          Response::Text(_) => 
            self.user.keys.get_text_dialog_action(&kc)
              .map(|a| Message::Action(a)),
          Response::Select(_) => 
            self.user.keys.get_select_dialog_action(&kc)
              .map(|a| Message::Action(a)),
        }
        Focus::Tab => 
          self.user.keys.get_tab_action(&kc)
            .map(|a| Message::Action(a)),
      }
      _ => None,
    }
  }
  fn banner_text(&self) -> String {
    let text = self.tabs.banner_text();
    if self.pending {
      format!(" (pending response) {} ", text)
    } else {text}
  }
  fn write(&self, stdout: &mut Stdout) -> io::Result<()> {
    cursor_hide(stdout)?;
    if self.clear {
      stdout.queue(Clear(ClearType::All))?;
      self.frame.write(stdout)?;
    }
    self.frame.write_banner(&self.banner_text(), stdout)?;
    if let Focus::Dialog(_, dialog) = &self.focus {
      if self.new_dlg {
        if let Some(fg) = self.user.style.covered.fg {
          self.tabs.write_style(&self.user.style.covered, stdout)?;
        } else {
          self.tabs.clear(stdout)?;
        }
      }
      dialog.prompt.write(stdout)?;
      match &dialog.response {
        Response::Ack(r) | Response::Ask(r) => 
          r.write(stdout)?,
        Response::Text(r) => {
          r.write(stdout)?;
          r.write_cursor(stdout)?;
        }
        Response::Select(r) => {
          r.write(stdout)?;
          r.write_cursor(stdout)?;
        }
      }
    } else {
      self.tabs.write(stdout)?;
      self.tabs.write_cursor(stdout)?;
    }
    stdout.flush()
  }
}
