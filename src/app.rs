// src/app.rs

use crate::{
  cursor::UnitCursor,
  user::{self, User, UserTable},
  keys::Action,
  view::Rect,
  tab::{Tab, TabList},
  widget::{Frame, TextBox},
  dialog::{Response, Dlg},
  protocol::{Request, GemDoc, GemTag, Status, Scheme},
};
use crossterm::{
  QueueableCommand, cursor,
  terminal::{Clear, ClearType},
  event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use url::Url;
use std::{fs, str::FromStr, io::{self, Write, Stdout}};


pub const MANUAL:        &str = "User manual";
pub const CHANGE_KEYS:   &str = "Change keys";
pub const CHANGE_STYLE:  &str = "Change style";
pub const VIEW_SETTINGS: &str = "View settings";
pub const MENU: [&str; 4] = [
  MANUAL, 
  CHANGE_KEYS, 
  CHANGE_STYLE,
  VIEW_SETTINGS, 
];

#[derive(Clone, Debug)]
pub enum Task {
  Default, 
  Reply,
  NewTab,
  DelTab,
  LoadUrl,
  Menu,
  ChangeStyle,
  ChangeKeys,
  Input(String),
  Redirect(String),
  Go(String), 
}

#[derive(Clone, Debug)]
pub enum Msg {
  Quit,
  Action(Action),
  Resize(u16, u16),
}

pub enum Focus {
  Tab, Dlg(Task, Dlg),
}

pub struct App {
  pub init_path:   String,
  pub frame:       Frame,
  pub user:        User,
  pub urls:        Vec<String>,
  pub screen:      Rect,
  pub rect:        Rect,
  pub tabs:        TabList,
  pub focus:       Focus,
  pub new_dlg:     bool,
  pub request:     Option<Request>,
  pub clear:       bool,
  pub guide:       String,
  pub tab_changed: bool,
  pub quit:        bool,
} 
impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {
    let user_text = fs::read_to_string(path).unwrap_or("".into());
    let user      = User::from_str(&user_text).unwrap_or_default();
    let screen    = Rect::new(w, h);
    let frame     = user.get_frame(&screen);
    let rect      = frame.inner_rect.clone();
    let tab       = Tab::init(&rect, &user.init_url);
    let urls: Vec<String> = match fs::read_to_string(&user.save_file) {
      Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
      Err(e) => vec![],
    };
    let mut app = Self {
      screen,
      frame,
      user,
      rect,
      urls,
      guide:       "".into(),
      init_path:   path.into(),
      tabs:        TabList::new(tab),
      request:     None,
      focus:       Focus::Tab,
      new_dlg:     false,
      tab_changed: true,
      clear:       true,
      quit:        false,
    };
    app.focus_tabs();
    app.try_spawn_request(&app.user.init_url.clone());
    app
  }
  pub fn focus_tabs(&mut self) {
    self.focus = Focus::Tab;
    self.guide = format!("Press {} for menu", self.user.keys.menu);
  }
  fn focus_ack_dlg(&mut self, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("Press any key to acknowledge");
    self.focus   = Focus::Dlg(Task::Default, Dlg::ack(prompt, help, style, &self.rect));
    self.guide   = format!("Press any key to acknowledge");
    self.new_dlg = true;
  }
  fn focus_ask_dlg(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    let help     = &format!("{} yes {} no", self.user.keys.yes, self.user.keys.no);
    self.focus   = Focus::Dlg(task, Dlg::ask(prompt, help, style, &self.rect));
    self.guide   = format!("{} yes {} no", self.user.keys.yes, self.user.keys.no);
    self.new_dlg = true;
  }
  fn focus_edit_dlg(&mut self, task: Task, prompt: &str) {
    let style    = self.user.style.info.style.clone();
    self.focus   = Focus::Dlg(task, Dlg::edit(prompt, style, &self.rect));
    self.guide   = format!("Press {} to cancel", self.user.keys.cancel);
    self.new_dlg = true;
  }
  fn focus_select_dlg(&mut self, task: Task, prompt: &str, options: Vec<String>) {
    let style    = self.user.style.info.style.clone();
    self.focus   = Focus::Dlg(task, Dlg::select(prompt, options, style, &self.rect));
    self.guide   = format!("Press {} to select", self.user.keys.select);
    self.new_dlg = true;
  }
  fn set_gemdoc(&mut self, url: &Url, gemdoc: GemDoc) {
    self.tabs.add(url.as_str());
    self.tab_changed = true;
    match gemdoc.status.tag {
      Status::InputExpected | Status::InputExpectedSensitive => {
        self.focus_edit_dlg(Task::Reply, &gemdoc.status.text);
      }
      Status::RedirectTemporary | Status::RedirectPermanent => {
        self.tabs.url_str = gemdoc.status.text.clone();
        self.focus_ask_dlg(Task::Redirect(gemdoc.status.text.clone()), &gemdoc.status.text);
      }
      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {
        self.focus_ack_dlg(&gemdoc.status.text);
      }
      _ => {}
    };
    self.tabs.content = TextBox::new(
        gemdoc.doc.iter().map(|gem| self.user.gem_to_styled(gem)).collect(),
        &self.rect,
      )
      .with_style(&self.user.style.general.style);
    self.tabs.gemdoc = Some(gemdoc);
  }
  pub fn try_join_request(&mut self) -> bool {
    if let Some(request) = &mut self.request {
      if request.handle.is_finished() {
        let result = request.rx
          .recv()
          .unwrap()
          .map(|(r, c)| GemDoc::new(&request.url, r, c))
          .flatten();
        match result {
          Err(e)     => self.focus_ack_dlg(&e),
          Ok(gemdoc) => {
            let url = request.url.clone();
            self.set_gemdoc(&url, gemdoc);
          }
        }
        self.request = None;
        true
      } else {false}
    } else {false}
  }
  pub fn try_spawn_request(&mut self, url_str: &str) {
    match Url::parse(url_str) {
      Err(e)  => {
        self.focus_ack_dlg(&format!("URL parse error: {}", &e)); 
        self.request = None;
      }
      Ok(url) => match &mut self.request {
        Some(_) => self.focus_ack_dlg("still processing previous request"),
        None    => self.request = Some(Request::new(&url, self.user.timeout)),
      }
    }
  }
  pub fn push_style(&mut self) {
    self.frame = self.user.get_frame(&self.screen);
    self.rect  = self.frame.inner_rect;
    for tab in self.tabs.tabs.iter_mut() {
      if let Some(gem) = &tab.gemdoc {
        tab.content.restyle(
            gem.doc.iter().map(|gem| self.user.gem_to_styled(gem)).collect(),
            &self.rect,
          )
      }
      tab.content.style = self.user.style.general.style.clone();
    }
    self.clear = true;
  }
  pub fn update(&mut self, message: &Msg) {
    self.clear       = false;
    self.tab_changed = false;
    self.new_dlg     = false;
    self.tabs.reset_state();
    match (message, &mut self.focus) {
      (Msg::Quit, _) => {
        self.quit = true;
      }
      (Msg::Resize(w, h), focus) => {
        self.screen = Rect::new(*w, *h);
        self.frame.resize(&self.screen);
        self.rect = self.frame.inner_rect.clone();
        if let Focus::Dlg(_, dialog) = focus {
          dialog.resize(&self.rect);
        }
        self.tabs.resize(&self.rect);
        self.clear = true;
      }
      (Msg::Action(action), Focus::Dlg(task, dlg)) 
        => match (&mut dlg.response, action, task) 
      {
        (Response::Select(textbox), Action::Select, Task::NewTab) => {
          if self.urls.len() > 0 {
            let url_str = &self.urls[textbox.get_source_idx()].clone();
            self.focus_tabs();
            self.try_spawn_request(url_str);
          } else {
            self.focus_tabs();
          }
        }
        (Response::Select(textbox), Action::Select, Task::ChangeKeys) => {
          match fs::read_to_string(user::get_keys_file(&textbox.get_source())) {
            Err(e) => self.focus_ack_dlg(&format!("Problem: {}", &e)),
            Ok(s) => if let Err(e) = self.user.keys.update_from_str(&s) {
              self.focus_ack_dlg(&format!("Problem: {}", &e));
            } else {
              self.focus_tabs();
            }
          }
        }
        (Response::Select(textbox), Action::Select, Task::ChangeStyle) => {
          match fs::read_to_string(user::get_styles_file(&textbox.get_source())) {
            Err(e) => self.focus_ack_dlg(&format!("Problem: {}", &e)),
            Ok(s) => if let Err(e) = self.user.style.update_from_str(&s) {
              self.focus_ack_dlg(&format!("Problem: {}", &e));
              self.push_style();
            } else {
              self.focus_tabs();
              self.push_style();
            }
          }
        }
        (Response::Select(textbox), Action::Select, Task::Menu) => {
          match MENU[textbox.get_source_idx()] {
            MANUAL => {
              self.focus_ack_dlg("View manual");
            }
            CHANGE_KEYS => match user::get_entries(user::KEYS_PATH) {
              Ok(e)  => self.focus_select_dlg(Task::ChangeKeys, "Choose keys", e),
              Err(e) => self.focus_ack_dlg(&format!("Problem: {}", &e)),
            }
            CHANGE_STYLE => match user::get_entries(user::STYLES_PATH) {
              Ok(e)  => self.focus_select_dlg(Task::ChangeStyle, "Choose style", e),
              Err(e) => self.focus_ack_dlg(&format!("Problem: {}", &e)),
            }
            VIEW_SETTINGS => {
              let text = format!("{:#?}", self.user).lines().map(|s| s.into()).collect();
              self.focus_select_dlg(Task::Default, "Current Settings", text);
            }
            _ => self.focus_tabs(),
          }
        }
        (Response::Edit(editbox), Action::Enter, Task::Reply) => {
          let text = editbox.content.to_string();
          let text = text.trim().replace(" ", "%20");
          self.focus_tabs();
          self.tab_changed = true;
          self.try_spawn_request(&format!("{}?{}", self.tabs.url_str, text));
        }
        (Response::Edit(editbox), Action::Enter, Task::NewTab) => {
          let text = editbox.content.to_string();
          self.focus_tabs();
          self.tab_changed = true;
          self.try_spawn_request(&text);
        }
        (_, Action::Yes, Task::Go(url)) => {
          let url = url.clone();
          self.focus_tabs();
          self.try_spawn_request(&url);
        }
        (_, Action::Yes, Task::Redirect(url_str)) => {
          let text = url_str.trim().replace(" ", "%20");
          self.focus_tabs();
          self.try_spawn_request(&format!("{}?{}", self.tabs.url_str, text));
        }
        (_, Action::Yes, Task::DelTab) => {
          self.tabs.delete();
          self.tab_changed = true;
          self.focus_tabs();
        }
        (Response::Ack(_), _, _) |
        (_,   Action::Select, _) |
        (_,       Action::No, _) |
        (_,   Action::Cancel, _)               => self.focus_tabs(),
        (Response::Select(textbox), action, _) => textbox.update(action),
        (Response::Edit(editbox),   action, _) => editbox.update(action),
        (_, _, _)                              => self.focus_tabs(),
      }
      (Msg::Action(action), Focus::Tab) => match action {
        Action::SaveUrl => {
          let url_str = self.tabs.url_str.clone();
          // only add url_str if new
          if self.urls.iter().any(|url| **url == url_str) {
            self.focus_ack_dlg(&format!("URL {} already saved", url_str)); 
          } else {
            self.urls.push(url_str.clone());
            // write to save file
            match fs::OpenOptions::new()
              .write(true).truncate(true).open(&self.user.save_file) 
            {
              Err(e) => 
                self.focus_ack_dlg(&format!("could not create save file: {}", &e)),
              Ok(mut f) => {
                for url in self.urls.iter() {
                  f.write(&format!("{}\n", url).as_bytes());
                }
                self.focus_ack_dlg(&format!("Saved URL: {}", url_str)); 
              }
            }
          }
        }
        Action::Select => if let Some(gemdoc) = &self.tabs.gemdoc {
          match gemdoc.doc[self.tabs.get_source_idx()].tag.clone() {
            GemTag::Link(Scheme::Gemini, url) => {
              let prompt = &format!("go to {}?", url);
              self.focus_ask_dlg(Task::Go(url.into()), prompt);
            }
            GemTag::Link(_, url) => 
              self.focus_ack_dlg(&format!("Protocol {} not yet supported", url)),
            gemtext => 
              self.focus_ack_dlg(&format!("you've selected {:?}", gemtext)),
          }
        }
        Action::CycleLeft => {
          if self.tabs.units().len() > 1 {
            self.tabs.wrapping_backward(1);
            self.tab_changed = true;
          }
        }
        Action::CycleRight => {
          if self.tabs.units().len() > 1 {
            self.tabs.wrapping_forward(1);
            self.tab_changed = true;
          }
        }
        Action::LoadUrl => 
          self.focus_select_dlg(Task::NewTab, "Choose URL: ", self.urls.clone()),
        Action::Menu => 
          self.focus_select_dlg(Task::Menu, "Choose: ", 
            MENU.iter().map(|s| s.to_string()).collect()),
        Action::NewTab => 
          self.focus_edit_dlg(Task::NewTab, "enter path: "),
        Action::DelTab => 
          self.focus_ask_dlg(Task::DelTab, "Delete current tab?"),
        action => 
          self.tabs.update(action),
      }
    }
  }
  pub fn get_update(&self, event: Event) -> Option<Msg> {
    match event {
      Event::Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c'), ..
      }) => 
        Some(Msg::Quit),
      Event::Resize(w, h) => 
        Some(Msg::Resize(w, h)),
      Event::Key(KeyEvent {
        code: kc, kind: KeyEventKind::Press, ..
      }) => match &self.focus {
        Focus::Tab => 
          self.user.keys.get_tab_action(&kc).map(|a| Msg::Action(a)),
        Focus::Dlg(_, dlg) => 
          self.user.keys.get_dlg_action(dlg, &kc).map(|a| Msg::Action(a))
      }
      _ => None,
    }
  }
  pub fn write(&self, stdout: &mut Stdout) -> io::Result<()> {
    stdout.queue(cursor::Hide)?;
    if self.clear {
      stdout.queue(Clear(ClearType::All))?;
      self.frame.write(stdout)?;
    }
    let banner_text = {
      let text = self.tabs.banner_text();
      if let Some(request) = &self.request {
        format!("(pending response) {}", text)
      } else {text}
    };
    self.frame.write_banner(&banner_text, stdout)?;
    self.frame.write_footer(&self.guide, &self.user.style.info.style, stdout)?;
    if let Focus::Dlg(_, dialog) = &self.focus {
      if self.new_dlg {
        self.tabs.clear(stdout)?;
      }
      dialog.write(stdout)?;
    } else {
      self.tabs.write(stdout)?;
      self.tabs.cursor.write(stdout)?;
    }
    stdout.flush()
  }
}
