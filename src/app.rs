// src/app.rs

use crate::{
  cursor::UnitCursor,
  user::{self, User, UserTable},
  keys::Action,
  view::{Rect, ViewPort},
  tab::{Tab, TabList},
  widget::{Frame, TextBox},
  dialog::{Response, Dialog},
  gemdoc::{self, GemTag, GemText, Status, StatusText},
  network::Request,
  util,
};
use crossterm::{
  QueueableCommand, cursor,
  terminal::{Clear, ClearType},
  event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use url::Url;
use std::{
  str::FromStr, 
  io::{Write, Stdout}
};


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
  NewTab,
  DelTab,
  LoadUrl,
  Menu,
  ChangeStyle,
  ChangeKeys,
  Input(String),
  Reply(Url),
  Go(Url), 
}

#[derive(Clone, Debug)]
pub enum Msg {
  Quit,
  Action(Action),
  Resize(u16, u16),
}

pub enum Focus {
  Tab, Dialog(Task, Dialog),
}

pub struct App {
  pub user:        User,
  pub frame:       Frame,
  pub tabs:        TabList,
  pub focus:       Focus,
  pub request:     Option<Request>,
  pub guide:       String,
  pub new_dlg:     bool,
  pub clear:       bool,
  pub tab_changed: bool,
  pub quit:        bool,
} 
impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {
    let user_text = std::fs::read_to_string(path).unwrap_or_default();
    let user      = User::from_str(&user_text).unwrap_or_default();
    let frame     = user.get_frame(Rect::new(w, h));
    let tab       = Tab::init(frame, &Url::parse(&user.init_url).unwrap());
    let mut app = Self {
      frame,
      user,
      guide:       "".into(),
      tabs:        TabList::new(tab),
      request:     None,
      focus:       Focus::Tab,
      new_dlg:     false,
      tab_changed: true,
      clear:       true,
      quit:        false,
    };
    app.focus_tabs();
    app.spawn_request(&app.tabs.url.clone());
    app
  }

  pub fn focus_tabs(&mut self) {
    self.focus = Focus::Tab;
    self.guide = format!("Press {} for menu", self.user.keys.menu);
  }

  fn focus_ack_dialog(&mut self, prompt: String) {
    let dlg = Dialog::ack(
      self.frame,
      self.user.style.info, 
      &prompt, 
      &format!("Press any key to acknowledge"), 
    );
    self.focus = Focus::Dialog(Task::Default, dlg);
    self.guide = format!("Press any key to acknowledge");
    self.new_dlg = true;
  }

  fn focus_ask_dialog(&mut self, task: Task, prompt: &str) {
    let dlg = Dialog::ask(
      self.frame,
      self.user.style.info, 
      prompt, 
      &format!("{} yes {} no", self.user.keys.yes, self.user.keys.no),
    );
    self.focus = Focus::Dialog(task, dlg);
    self.guide = 
      format!("{} yes {} no", self.user.keys.yes, self.user.keys.no);
    self.new_dlg = true;
  }

  fn focus_edit_dialog(&mut self, task: Task, prompt: &str) {
    self.focus = Focus::Dialog(
      task, 
      Dialog::edit(self.frame, self.user.style.info, prompt)
    );
    self.guide = format!("Press {} to cancel", self.user.keys.cancel);
    self.new_dlg = true;
  }

  fn focus_select_dialog(
    &mut self, 
    task: Task, 
    prompt: &str, 
    options: Vec<String>
  ) {
    let dlg = Dialog::select(
      self.frame,
      self.user.style.info, 
      prompt, 
      options, 
    );
    self.guide = format!("Press {} to select", self.user.keys.select);
    self.focus = Focus::Dialog(task, dlg);
    self.new_dlg = true;
  }

  fn join_gemdoc(&mut self, url: &Url, response: String, content: String) {
    let Ok(status) = StatusText::try_from(response.as_str()) else {
      self.focus_ack_dialog(format!(
        "Response {} is not valid for gemini protocol", response
      ));
      return
    };
    match status.tag {
      Status::InputExpected | 
      Status::InputExpectedSensitive => {
        self.focus_edit_dialog(
          Task::Reply(url.clone()), 
          &status.text
        );
      }
      Status::RedirectTemporary | 
      Status::RedirectPermanent => match Url::parse(&status.text) {
        Err(e) => self.focus_ack_dialog(format!(
          "Redirects to invalid URL. {}", e
        )),
        Ok(url) => self.focus_ask_dialog(
          Task::Go(url.clone()), 
          &status.text
        ),
      }
      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {
        self.focus_ack_dialog(status.text);
      }
      _ => {
        self.tabs.add(url);
        let doc = gemdoc::parse_doc(&content);
        self.tabs.content = TextBox::new(
            self.frame,
            doc
              .iter()
              .map(|gem| self.user.get_styled_gemtext(gem))
              .collect(),
          ).with_style(self.user.style.general);
        self.tabs.source = doc;
        self.tab_changed = true;
      }
    };
  }

  pub fn join_request(&mut self) -> bool {
    let Some(request) = &mut self.request else {
      return false
    };
    if !request.handle.is_finished() {
      return false
    }
    match request.rx.recv().unwrap() {
      Err(e) => {
        self.focus_ack_dialog(e);
        self.request = None;
        true
      }
      Ok((r, c)) => {
        let url = request.url.clone();
        self.join_gemdoc(&url, r, c);
        self.request = None;
        true
      }
    }
  }

  pub fn spawn_request(&mut self, url: &Url) {
    match (&mut self.request, url.scheme()) {
      (None, "gemini") => self.request = Some(
        Request::new(&url, self.user.timeout)
      ),
      (None, scheme) => self.focus_ack_dialog(
        format!("Protocol {} not yet supported", scheme)
      ),
      (Some(request), _) => {
        let url = request.url.to_string();
        self.focus_ack_dialog(
          format!("still processing request for {}", url)
        );
      }
    }
  }

  pub fn push_style(&mut self) {
    self.frame = self.user.get_frame(self.frame.screen);
    for tab in self.tabs.tabs.iter_mut() {
      let source = &tab.source;
      tab.content.restyle(
        self.frame,
        source
          .iter()
          .map(|gem| self.user.get_styled_gemtext(gem))
          .collect(),
      );
      tab.content.style = self.user.style.general.into();
    }
    self.clear = true;
  }

  pub fn select_link(&mut self, url_str: &str) {
    match util::join_if_relative(&self.tabs.url, url_str) {
      Err(e) => self.focus_ack_dialog(format!(
        "{} -- Invalid URL. {}", url_str, e)),
      Ok(url) => {
        let prompt = &format!("{} -- Make request?", url);
        self.focus_ask_dialog(Task::Go(url.into()), prompt);
      } 
    }
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
        self.frame.resize(Rect::new(*w, *h));
        if let Focus::Dialog(_, dialog) = focus {
          dialog.resize(self.frame);
        }
        self.tabs.resize(self.frame);
        self.clear = true;
      }
      (Msg::Action(action), Focus::Dialog(task, dlg)) 
        => match (&mut dlg.response, action, task) 
      {
        (Response::Select(textbox), Action::Select, Task::NewTab) => {
          if self.user.urls.len() > 0 {
            let link = &self.user.urls[textbox.get_source_idx()].clone();
            self.select_link(link);
          } else {
            self.focus_tabs();
          }
        }
        (Response::Select(textbox), Action::Select, Task::ChangeKeys) => {
          match std::fs::read_to_string(
            user::get_keys_file(&textbox.get_source())) 
          {
            Err(e) => self.focus_ack_dialog(format!("Problem: {}", &e)),
            Ok(s)  => if let Err(e) = self.user.keys.update_from_str(&s) {
              self.focus_ack_dialog(format!("Problem: {}", &e));
            } else {
              self.focus_tabs();
            }
          }
        }
        (Response::Select(textbox), Action::Select, Task::ChangeStyle) => {
          match std::fs::read_to_string(
            user::get_styles_file(&textbox.get_source())) 
          {
            Err(e) => self.focus_ack_dialog(e.to_string()),
            Ok(s)  => if let Err(e) = self.user.style.update_from_str(&s) {
              self.focus_ack_dialog(e.to_string());
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
              self.focus_ack_dialog("View manual".into());
            }
            CHANGE_KEYS => match util::get_entries(user::KEYS_PATH) {
              Err(e)    => self.focus_ack_dialog(e),
              Ok(entry) => self.focus_select_dialog(
                Task::ChangeKeys, "Choose keys", entry
              ),
            }
            CHANGE_STYLE => match util::get_entries(user::STYLES_PATH) {
              Err(e)    => self.focus_ack_dialog(e),
              Ok(entry) => self.focus_select_dialog(
                Task::ChangeStyle, "Choose style", entry
              ),
            }
            VIEW_SETTINGS => {
              let text = format!("{:#?}", self.user)
                .lines()
                .map(|s| s.into())
                .collect();
              self.focus_select_dialog(
                Task::Default, "Current Settings", text
              );
            }
            _ => self.focus_tabs(),
          }
        }
        (Response::Edit(editbox), Action::Enter, Task::Reply(url)) => {
          let text = editbox.content.to_string().trim().replace(" ", "%20");
          match url.clone().join(&format!("?{}", text)) {
            Err(e) => self.focus_ack_dialog(
              format!("Invalid URL. {}", e)),
            Ok(url) => {
              self.focus_tabs();
              self.tab_changed = true;
              self.spawn_request(&url);
            }
          }
        }
        (Response::Edit(editbox), Action::Enter, Task::NewTab) => {
          match Url::parse(&editbox.content.to_string()) {
            Err(e) => self.focus_ack_dialog(format!(
              "Invalid URL. {}", e)),
            Ok(url) => {
              self.focus_tabs();
              self.tab_changed = true;
              self.spawn_request(&url);
            }
          }
        }
        (_, Action::Yes, Task::Go(url)) => {
          let url = url.clone();
          self.focus_tabs();
          self.spawn_request(&url);
        }
        (_, Action::Yes, Task::DelTab) => {
          self.tabs.delete();
          self.tab_changed = true;
          self.focus_tabs();
        }
        (Response::Ack(_), _, _) |
        (_,   Action::Select, _) |
        (_,       Action::No, _) |
        (_,   Action::Cancel, _) => {
          self.focus_tabs();
        }
        (Response::Select(textbox), action, _) => {
          textbox.update(action);
        }
        (Response::Edit(editbox),   action, _) => {
          editbox.update(action);
        }
        (_, _, _) => {
          self.focus_tabs();
        }
      }
      (Msg::Action(Action::SaveUrl), Focus::Tab) => {
        match self.user.save_url(&self.tabs.url) {
          Err(e) => self.focus_ack_dialog(e),
          Ok(()) => self.focus_ack_dialog(format!(
            "Saved URL: {}", self.tabs.url.to_string())),
        }
      }
      (Msg::Action(Action::Select), Focus::Tab) => {
        match self.tabs.source[self.tabs.get_source_idx()].tag.clone() {
          GemTag::Link(link) => {
            let link = link.clone();
            self.select_link(&link);
          }
          gemtext => self.focus_ack_dialog(format!(
            "You've selected {:?}", gemtext)
          ),
        }
      }
      (Msg::Action(Action::CycleLeft), Focus::Tab) => {
        if self.tabs.units().len() > 1 {
          self.tabs.wrapping_backward(1);
          self.tab_changed = true;
        }
      }
      (Msg::Action(Action::CycleRight), Focus::Tab) => {
        if self.tabs.units().len() > 1 {
          self.tabs.wrapping_forward(1);
          self.tab_changed = true;
        }
      }
      (Msg::Action(Action::LoadUrl), Focus::Tab) => {
        self.focus_select_dialog(
          Task::NewTab, 
          "Choose URL: ", 
          self.user.urls.clone());
      }
      (Msg::Action(Action::Menu), Focus::Tab) => {
        self.focus_select_dialog(
          Task::Menu, 
          "Choose: ", 
          MENU.iter().map(|s| s.to_string()).collect());
      }
      (Msg::Action(Action::NewTab), Focus::Tab) => {
        self.focus_edit_dialog(Task::NewTab, "enter path: ");
      }
      (Msg::Action(Action::DelTab), Focus::Tab) => {
        self.focus_ask_dialog(Task::DelTab, "Delete current tab?");
      }
      (Msg::Action(action), Focus::Tab) => {
        self.tabs.update(action);
      }
    }
  }

  pub fn get_update(&self, event: Event) -> Option<Msg> {
    match event {
      Event::Resize(w, h) => {
        Some(Msg::Resize(w, h))
      }
      Event::Key(
        KeyEvent {
          modifiers: KeyModifiers::CONTROL, 
          code:      KeyCode::Char('c'), 
          ..
        }
      ) => {
        Some(Msg::Quit)
      }
      Event::Key(
        KeyEvent {
          kind: KeyEventKind::Press, 
          code: kc, 
          ..
        }
      ) => match &self.focus {
        Focus::Dialog(_, dlg) => 
          self.user.keys.get_dlg_action(dlg, &kc).map(Msg::Action),
        Focus::Tab => 
          self.user.keys.get_tab_action(&kc).map(Msg::Action),
      }
      _ => None,
    }
  }

  pub fn write(&self, stdout: &mut Stdout) -> std::io::Result<()> {
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
    self.frame.write_footer(&self.guide, stdout)?;
    if let Focus::Dialog(_, dialog) = &self.focus {
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
