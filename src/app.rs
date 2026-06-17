// src/app.rs

use crate::{
  gemini, 
  util,
  user,
  User, 
  UserTable,
  UserFromStr,
  TabManager,
  Request,
  DlgInput, 
  Draw,
  Dialog,
  Action,
  Rect, 
  GemTag, 
  Status, 
  StatusText,
  Frame,
  ViewStack,
};
use url::Url;


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

#[derive(Debug)]
pub enum Task {
  Default, 
  NewTab,
  DelTab,
  LoadUrl,
  Menu,
  ChangeStyle,
  ChangeKeys,
  Init(String),
  Reply(Url),
  Go(Url), 
}

#[derive(Copy, Clone, Debug)]
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
  pub tabs:        TabManager,
  pub focus:       Focus,
  pub request:     Option<Request>,
  pub guide:       String,
  pub viewstack:   ViewStack,
  pub new_dlg:     bool,
  pub clear:       bool,
  pub tab_changed: bool,
  pub quit:        bool,
} 

impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {
    let user_text = std::fs::read_to_string(path).unwrap_or_default();
    let user      = User::user_from_str(&user_text).unwrap_or_default();
    let frame     = user.get_frame(Rect::new(w, h));
    let mut app = Self {
      guide:       "".into(),
      tabs:        TabManager::from(frame).with_style(user.style.general),
      viewstack:   ViewStack::from(frame),
      request:     None,
      focus:       Focus::Tab,
      new_dlg:     false,
      tab_changed: true,
      clear:       true,
      quit:        false,
      frame,
      user,
    };
    match Url::parse(&app.user.init_url) {
      Err(e) => app.focus_edit_dialog(
        Task::Init(app.user.init_url.clone()), 
        &format!("Try again: {e}"), 
        &app.user.init_url.clone(),
      ),
      Ok(url) => {
        app.focus_tabs();
        app.spawn_request(&url);
      }
    }
    app
  }

  pub fn focus_tabs(&mut self) {
    self.focus = Focus::Tab;
    self.guide = format!("Press {} for menu", self.user.keys.menu);
  }

  fn focus_ack_dialog(&mut self, prompt: String) {
    self.guide = format!("Press any key to acknowledge");
    let dlg = Dialog::ack(
      self.frame,
      self.user.style.info, 
      &prompt, 
      &self.guide, 
    );
    self.focus = Focus::Dialog(Task::Default, dlg);
    self.new_dlg = true;
  }

  fn focus_ask_dialog(&mut self, task: Task, prompt: &str) {
    self.guide = format!(
      "{} yes {} no", self.user.keys.yes, self.user.keys.no
    );
    let dlg = Dialog::ask(
      self.frame,
      self.user.style.info, 
      prompt, 
      &self.guide,
    );
    self.focus = Focus::Dialog(task, dlg);
    self.new_dlg = true;
  }

  fn focus_edit_dialog(&mut self, task: Task, prompt: &str, text: &str) {
    self.guide = format!("Press {} to cancel", self.user.keys.cancel);
    self.focus = Focus::Dialog(
      task, 
      Dialog::edit(self.frame, self.user.style.info, prompt, text)
    );
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
      self.focus_ack_dialog(
        format!("Response {response} is not valid for gemini protocol")
      );
      return
    };
    match status.tag {
      Status::InputExpected | 
      Status::InputExpectedSensitive => {
        self.focus_edit_dialog(
          Task::Reply(url.clone()), 
          &status.text, ""
        );
      }
      Status::RedirectTemporary | 
      Status::RedirectPermanent => match Url::parse(&status.text) {
        Err(e) => self.focus_ack_dialog(
          format!("Redirects to invalid URL. {e}")
        ),
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
        self.tabs.add_gem_tab(
          url, 
          gemini::parse_doc(&content), 
          |g| self.user.get_styled_gemtext(g),
        );
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
        format!("Protocol {scheme} not yet supported")
      ),
      (Some(request), _) => {
        let url = request.url.to_string();
        self.focus_ack_dialog(
          format!("still processing request for {url}")
        );
      }
    }
  }

  pub fn push_size(&mut self) {
    if let Focus::Dialog(_, dialog) = &mut self.focus {
      dialog.resize(self.frame);
    }
    self.tabs.resize(self.frame);
    self.clear = true;
  }

  pub fn push_style(&mut self) {
    self.frame = self.user.get_frame(self.frame.screen);
    self.push_size();
    self.tabs.push_style(self.user.style.general);
    self.tabs.push_gem_style(
      |gem| self.user.get_styled_gemtext(gem)
    );
  }

  pub fn select_link(&mut self, url_str: &str) {
    match self.tabs
      .get_url()
      .map(|url| util::join_if_relative(&url, url_str)) 
    {
      None => {},
      Some(Err(e)) => self.focus_ack_dialog(
        format!("{url_str} -- Invalid URL. {e}")
      ),
      Some(Ok(url)) => {
        let prompt = &format!("{url} -- Make request?");
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
        self.push_size();
      }
      (Msg::Action(action), Focus::Dialog(task, dlg)) 
        => match (&mut dlg.input, action, task) 
      {
        (DlgInput::Select(textbox), Action::Select, Task::NewTab) => {
          if let Some(link) = self.user.urls.get(
            textbox.get_current_reference_index()
          ) {
            let link = link.clone();
            self.select_link(&link);
          } else {
            self.focus_tabs();
          }
        }
        (DlgInput::Select(textbox), Action::Select, Task::ChangeKeys) => {
          match std::fs::read_to_string(
            user::get_keys_file(
              &textbox.get_current_display_ref()
            )
          ) {
            Err(e) => self.focus_ack_dialog(format!("Problem: {e}")),
            Ok(s)  => if let Err(e) = self.user.keys.update_from_str(&s) {
              self.focus_ack_dialog(format!("Problem: {e}"));
            } else {
              self.focus_tabs();
            }
          }
        }
        (DlgInput::Select(textbox), Action::Select, Task::ChangeStyle) => {
          match std::fs::read_to_string(
            user::get_styles_file(
              &textbox.get_current_display_ref()
            )
          ) {
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
        (DlgInput::Select(textbox), Action::Select, Task::Menu) => {
          match MENU[textbox.get_current_reference_index()] {
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
        (DlgInput::Edit(editbox), Action::Enter, Task::Init(_)) => {
          let url_str = editbox.get_current_string().unwrap();
          match Url::parse(&url_str) {
            Err(e) => 
              self.focus_edit_dialog(
                Task::Init(url_str.clone()),
                &format!("Invalid URL. {}", e), 
                &url_str
              ),
            Ok(url) => {
              self.focus_tabs();
              self.tab_changed = true;
              self.spawn_request(&url);
            }
          }
        }
        (DlgInput::Edit(editbox), Action::Cancel, Task::Init(url_str)) => {
          let url_str = url_str.clone();
          self.focus_ask_dialog(
            Task::Init(url_str), "Exit application?".into()
          )
        }
        (DlgInput::Edit(editbox), Action::Enter, Task::Reply(url)) => {
          let text = editbox
            .get_current_string()
            .unwrap()
            .trim()
            .replace(" ", "%20");
          match url.clone().join(&format!("?{text}")) {
            Err(e) => self.focus_ack_dialog(
              format!("Invalid URL. {e}")
            ),
            Ok(url) => {
              self.focus_tabs();
              self.tab_changed = true;
              self.spawn_request(&url);
            }
          }
        }
        (DlgInput::Edit(editbox), Action::Enter, Task::NewTab) => {
          match Url::parse(
            &editbox.get_current_string().unwrap()
          ) {
            Err(e) => 
              self.focus_ack_dialog(
                format!("Invalid URL. {e}")
              ),
            Ok(url) => {
              self.focus_tabs();
              self.tab_changed = true;
              self.spawn_request(&url);
            }
          }
        }
        (_, Action::Cancel, Task::Init(url_str)) |
        (_, Action::No,     Task::Init(url_str)) => {
          let url_str = url_str.clone();
          self.focus_edit_dialog(
            Task::Init(url_str.clone()), 
            &format!("Enter URL: "), 
            &url_str
          );
        }
        (_, Action::Yes, Task::Init(_)) => {
          self.quit = true;
        }
        (_, Action::Yes, Task::Go(url)) => {
          let url = url.clone();
          self.focus_tabs();
          self.spawn_request(&url);
        }
        (_, Action::Yes, Task::DelTab) => {
          if self.tabs.remove() == 0 {
            let url_str = self.user.init_url.clone();
            self.focus_edit_dialog(
              Task::Init(url_str.clone()), 
              &format!("Enter URL: "), 
              &url_str
            );
          } else {
            self.tab_changed = true;
            self.focus_tabs();
          }
        }
        (DlgInput::Ack(_), _, _) |
        (_,   Action::Select, _) |
        (_,       Action::No, _) |
        (_,   Action::Cancel, _) => {
          self.focus_tabs();
        }
        (DlgInput::Select(textbox), action, _) => {
          textbox.update(action);
        }
        (DlgInput::Edit(editbox),   action, _) => {
          editbox.update_edit(action);
        }
        (_, _, _) => {
          self.focus_tabs();
        }
      }
      (Msg::Action(Action::SaveUrl), Focus::Tab) => {
        if let Some(url) = self.tabs.get_url() {
          match self.user.save_url(url) {
            Err(e) => self.focus_ack_dialog(e),
            Ok(()) => self.focus_ack_dialog(
              format!("Saved URL: {url}")
            ),
          }
        }
      }
      (Msg::Action(Action::Select), Focus::Tab) => {
        match self.tabs.use_gem_text(
          |gem_text| gem_text.tag.clone()
        ) {
          None => self.focus_ack_dialog(
            format!("You've selected nothing")
          ),
          Some(GemTag::Link(link)) => {
            let link = link.clone();
            self.select_link(&link);
          }
          Some(gemtag) => self.focus_ack_dialog(
            format!("You've selected {gemtag:?}")
          ),
        }
      }
      (Msg::Action(Action::CycleLeft), Focus::Tab) => {
        self.tab_changed = self.tabs.move_backward_wrapped(1);
      }
      (Msg::Action(Action::CycleRight), Focus::Tab) => {
        self.tab_changed = self.tabs.move_forward_wrapped(1);
      }
      (Msg::Action(Action::LoadUrl), Focus::Tab) => {
        self.focus_select_dialog(
          Task::NewTab, 
          "Choose URL: ", 
          self.user.urls.clone()
        );
      }
      (Msg::Action(Action::Menu), Focus::Tab) => {
        self.focus_select_dialog(
          Task::Menu, 
          "Choose: ", 
          MENU.iter().map(|s| s.to_string()).collect()
        );
      }
      (Msg::Action(Action::NewTab), Focus::Tab) => {
        self.focus_edit_dialog(
          Task::NewTab, "enter path: ", ""
        );
      }
      (Msg::Action(Action::DelTab), Focus::Tab) => {
        self.focus_ask_dialog(
          Task::DelTab, "Delete current tab?"
        );
      }
      (Msg::Action(action), Focus::Tab) => {
        self.tabs.use_textbox_mut(
          |textbox| textbox.update(action)
        );
      }
    }
  }

  pub fn get_update(&self, event: crossterm::event::Event) -> Option<Msg> {
    use crossterm::event;
    match event {
      event::Event::Resize(w, h) => {
        Some(Msg::Resize(w, h))
      }
      event::Event::Key(
        event::KeyEvent {
          modifiers: event::KeyModifiers::CONTROL, 
          code:      event::KeyCode::Char('c'), 
          ..
        }
      ) => {
        Some(Msg::Quit)
      }
      event::Event::Key(
        event::KeyEvent {
          kind: event::KeyEventKind::Press, 
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

  pub fn draw<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor, terminal};
    w.queue(cursor::Hide)?;
    if self.clear {
      w.queue(terminal::Clear(terminal::ClearType::All))?;
      self.frame.draw(w)?;
    }
    let banner_text = self.tabs.get_banner_text();
    self.frame.write_banner(&banner_text, w)?;
    self.frame.write_footer(&self.guide, w)?;
    if let Focus::Dialog(_, dialog) = &self.focus {
      dialog.draw(w)?;
    } else {
      if let Some(request) = &self.request {
    //  let tb: TextBox = TextBox::from(
    //      self.frame.get_view_port().top_row()
    //    ).reference(
    //      &vec![format!("requesting {}", request.url)],
    //      |s| StyledText::from(s.clone())
    //    );
    //  tb.draw(writer)?;
        self.tabs.draw(w)?;
      } else {
        self.tabs.draw(w)?;
      }
    }
    w.flush()
  }
}
