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

use crate::{
  common as c,
  user::{User, Action},
  tab::{Tab, TabList},
  dialog::{Response, ResponseType, Dialog},
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
  // will be Some after message_maybe is checked in the first loop
  let mut request_maybe: Option<Request> = None;
  // first message is Request, which will request the initial url
  let mut message_maybe: Option<Message> = app.get_init_url().map(|url| Message::Request(url));
  // initial display
  app.write(&mut stdout)?;
  // break on control-c
  loop {
    // inspect status of request
    if let Some(request) = &mut request_maybe {
      // handle is finished
      if request.handle.is_finished() {
        let result = request.rx.recv().unwrap()
          .map(|(r, c)| GemDoc::new(&request.url, r, c)).flatten();
        // get gemdoc
        match result {
          // no gemdoc, create error dialog
          Err(e)     => app.ack(Message::Default, &e),
          // process gemdoc
          Ok(gemdoc) => app.set_gemdoc(&request.url, gemdoc),
        }
        app.pending = false;
        app.write(&mut stdout)?;
        request_maybe = None;
      }
    } 
    // check for user input
    if event::poll(Duration::from_millis(16))? {
      message_maybe = app.update(event::read()?);
    } 
    // some update took place
    if let Some(message) = &message_maybe {
      // create new request if there isnt one already
      if let Message::Request(url) = message {
        // create new request
        if let None = &mut request_maybe {
          app.pending = true;
          request_maybe = Some(Request::new(&url, app.user.timeout));
        }
      // exit loop
      } else if let Message::Quit = message {
        break
      }
      // display new application state
      app.write(&mut stdout)?;
    }
    message_maybe = None;
  }
  // return terminal to normal state
  stdout
    .queue(terminal::LeaveAlternateScreen)?
    .queue(terminal::EnableLineWrap)?
    .queue(SetCursorStyle::DefaultUserShape)?
    .flush()?;
  terminal::disable_raw_mode()
}

#[derive(Clone, Debug)]
pub enum Message {
  Quit,
  Default, 
  CycleLeft, 
  CycleRight, 
  Delete, 
  NewTab, 
  Reply,
  Request(Url),
  Input(String),
  Redirect(String),
  Go(String), 
}
pub enum Focus {
  Tab, Dialog(ResponseType),
}

pub struct DialogTask {
  pub msg: Message,
  pub dlg: Dialog,
}

pub struct App {
  pub init_path: String,
  pub frame:     Frame,
  pub user:      User,
  pub urls:      Vec<String>,
  pub rect:      Rect,
  pub head:      usize,
  pub tabs:      TabList,
  pub dialog:    Option<(Message, Dialog)>,
  pub new_dlg:   bool,
  pub pending:   bool,
  pub clear:     bool,
} 
impl Linear for App {
  fn len(&self) -> usize {
    self.tabs.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
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
      head:      0,
      tabs:      TabList::new(tab),
      dialog:    None,
      new_dlg:   false,
      pending:   false,
      clear:     true
    }
  }
  fn ack(&mut self, msg: Message, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("Press {} to acknowledge", self.user.keys.ack);
    let dialog   = Dialog::ack(prompt, help, style, &self.rect);
    self.dialog  = Some((msg, dialog));
    self.new_dlg = true;
  }
  fn ask(&mut self, msg: Message, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("{} yes {} no", self.user.keys.yes, self.user.keys.no);
    let dialog   = Dialog::ask(prompt, help, style, &self.rect);
    self.dialog  = Some((msg, dialog));
    self.new_dlg = true;
  }
  fn text(&mut self, msg: Message, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let dialog   = Dialog::text(prompt, style, &self.rect);
    self.dialog  = Some((msg, dialog));
    self.new_dlg = true;
  }
  fn select_url(&mut self, msg: Message, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let dialog   = Dialog::select(prompt, self.urls.clone(), style, &self.rect);
    self.dialog  = Some((msg, dialog));
    self.new_dlg = true;
  }
  fn get_init_url(&mut self) -> Option<Url> {
    match Url::parse(&self.user.init_url) {
      Ok(url) => Some(url),
      Err(e) => {
        self.ack(Message::Default, &e.to_string());
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
        self.text(Message::Reply, &gemdoc.status.txt);
      }
      Status::RedirectTemporary | Status::RedirectPermanent => {
        self.tabs.url_str.push_str(&gemdoc.status.txt);
        self.ask(Message::Redirect(gemdoc.status.txt.clone()), &gemdoc.status.txt);
      }
      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {
        self.ack(Message::Default, &gemdoc.status.txt);
      }
      _ => {}
    };
    self.tabs.content = self.user.get_gem_textbox(&self.rect, &gemdoc);
    self.tabs.gemdoc  = Some(gemdoc);
  }
  fn update(&mut self, event: Event) -> Option<Message> {
    self.clear = false;
    self.new_dlg = false;
    match event {
      Event::Resize(w, h) => {
        self.frame.resize(&Rect::new(w, h));
        self.rect = self.frame.inner_rect.clone();
        if let Some((_, dialog)) = &mut self.dialog {
          dialog.resize(&self.rect);
        }
        self.tabs.resize(&self.rect);
        self.clear = true;
        Some(Message::Default)
      }
      Event::Key(
        KeyEvent {
          modifiers: KeyModifiers::CONTROL,
          code: KeyCode::Char('c'),
          kind: KeyEventKind::Press, ..
        }
      ) => {
        self.clear = true;
        Some(Message::Quit)
      }
      Event::Key(
        KeyEvent {
          code: kc, 
          kind: KeyEventKind::Press, ..
        }
      ) => {
        if let Some(response) = self.process_keycode(&kc) {
          match response {
            Message::Input(url_str) | Message::Go(url_str) => {
              match Url::parse(&url_str) {
                // failed, create error dialog
                Err(e)  => {
                  self.ack(Message::Default, &e.to_string());
                  Some(Message::Default)
                }
                Ok(url) => Some(Message::Request(url)),
              }
            }
            Message::Redirect(url_str) => {
              self.tabs.url_str = url_str.clone();
              match Url::parse(&url_str) {
                // failed, create error dialog
                Err(e)  => {
                  self.ack(Message::Delete, &e.to_string());
                  Some(Message::Default)
                }
                Ok(url) => Some(Message::Request(url)),
              }
            }
            Message::Delete => {
              self.tabs.delete();
              Some(Message::Default)
            }
            Message::CycleLeft => {
              if self.tabs.len() > 1 {
                self.wrapping_backward(1);
                Some(Message::Default)
              } else {None}
            }
            Message::CycleRight => {
              if self.tabs.len() > 1 {
                self.wrapping_forward(1);
                Some(Message::Default)
              } else {None}
            }
            _ => Some(Message::Default)
          } 
        } else {None}
      }
      _ => None
    }
  }
  fn get_key(&self, kc: &KeyCode) -> Option<Action> {
    let focus = match &self.dialog {
      Some((_, dialog)) => Focus::Dialog(dialog.response.get_type()),
      None => Focus::Tab,
    };
    self.user.keys.get(focus, kc)
  }
  fn process_keycode(&mut self, kc: &KeyCode) -> Option<Message> {
    self.tabs.reset_state();
    // process keycode for dialog
    if let Some((msg, dialog)) = &mut self.dialog {
      match kc {
        KeyCode::Esc => {
          self.dialog = None;
          Some(Message::Default)
        }
        _ => match &mut dialog.response {
          Response::Select(content) => {
            if self.user.keys.move_content(&kc, content) {
              Some(Message::Default)
            } else if kc == &self.user.keys.inspect {
              if let Message::NewTab = &msg {
                if self.urls.len() > 0 {
                  let msg = Some(Message::Go(self.urls[content.get_source_idx()].clone()));
                  self.dialog = None;
                  msg
                } else {
                  self.dialog = None;
                  Some(Message::Default)
                }
              } else {
                self.dialog = None;
                Some(Message::Default)
              }
            } else {None}
          }
          Response::Ack(_) => {
            if self.user.keys.ack == *kc {
              let msg = Some(msg.clone());
              self.dialog = None;
              msg
            } else {None}
          }
          Response::Ask(_) => {
            if self.user.keys.yes == *kc {
              let msg = Some(msg.clone());
              self.dialog = None;
              msg
            } else if self.user.keys.no == *kc {
              self.dialog = None;
              Some(Message::Default)
            } else {None}
          }
          Response::Text(editor) => {
            match kc {
              KeyCode::Enter => {
                let text = editor.content.text.to_string();
                let msg = 
                  if let Message::NewTab = msg {
                    Some(Message::Go(text))
                  } else if let Message::Reply = msg {
                    let text = text.trim().replace(" ", "%20");
                    let reply = format!("{}?{}", self.tabs.url_str, text);
                    Some(Message::Input(reply))
                  } else {
                    Some(msg.clone())
                  };
                self.dialog = None;
                msg
              }
              KeyCode::Left      => editor.left(1).then_some(Message::Default),
              KeyCode::Right     => editor.right(1).then_some(Message::Default),
              KeyCode::Delete    => editor.delete().then_some(Message::Default),
              KeyCode::Backspace => editor.backspace().then_some(Message::Default),
              KeyCode::Char(c) => {
                editor.insert(*c);
                Some(Message::Default)
              }
              _ => None
            }
          }
        }
      }
    // no dialog
    } else if self.user.keys.move_content(&kc, &mut self.tabs) {
      Some(Message::Default)
    } else if kc == &self.user.keys.cycle_left {
      Some(Message::CycleLeft)
    } else if kc == &self.user.keys.cycle_right {
      Some(Message::CycleRight)
    // make a dialog
    } else if kc == &self.user.keys.delete_tab {
      self.ask(Message::Delete, "Delete current tab?");
      Some(Message::Default)
    } else if kc == &self.user.keys.new_tab {
      self.text(Message::NewTab, "enter path: ");
      Some(Message::Default)
    } else if kc == &self.user.keys.load_url {
      self.select_url(Message::NewTab, "choose the url: ");
      Some(Message::Default)
    } else if kc == &self.user.keys.save_url {
      let url_str = self.tabs.url_str.clone();
      // only add url_str if new
      if !self.urls.iter().any(|url| **url == url_str) {
        self.urls.push(url_str);
        // write to save file
        match fs::OpenOptions::new().write(true).truncate(true).open(&self.user.save_file) {
          Err(e) => {
            self.ack(Message::Default, &format!("could not create save file: {}", &e));
            Some(Message::Default)
          }
          Ok(mut f) => {
            for url in self.urls.iter() {
              f.write(&format!("{}\n", url).as_bytes());
            }
            None
          }
        }
      } else {None}
    } else if kc == &self.user.keys.inspect {
      if let Some(gemdoc) = &self.tabs.gemdoc {
        match gemdoc.doc[self.tabs.get_source_idx()].tag.clone() {
          GemTag::Link(Scheme::Gemini, url) => {
            let prompt = &format!("go to {}?", url);
            self.ask(Message::Go(url.into()), prompt);
          }
          GemTag::Link(_, url) => {
            self.ack(Message::Default, &format!("Protocol {} not yet supported", url));
          }
          gemtext => {
            self.ack(Message::Default, &format!("you've selected {:?}", gemtext));
          }
        }
        Some(Message::Default)
      } else {None}
    } else {None}
  }
  fn write(&self, stdout: &mut Stdout) -> io::Result<()> {
    cursor_hide(stdout)?;
    if self.clear {
      stdout.queue(Clear(ClearType::All))?;
      self.frame.write(stdout)?;
    }
    self.frame.write_banner(&self.tabs.banner_text(), stdout)?;
    if let Some((_, dialog)) = &self.dialog {
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
