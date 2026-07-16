// src/app.rs

use crate::{
  gemini, 
  util,
  TabText,
  Dim,
  user,
  SystemParams, 
  Draw,
  CursorVec,
  AppView,
  Tab,
  TextParams,
  BuildView,
  DialogParams,
  DlgType,
  UserTable,
  user_from_str,
  Request,
  Action,
  Rect, 
  PageParams,
  GemTag, 
  Status, 
  StatusText,
  Frame,
  Resize,
  constants::*,
};


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
  Reply(url::Url),
  Go(url::Url), 
}

#[derive(Copy, Clone, Debug)]
pub enum Msg {
  Quit,
  Action(Action),
  Resize(u16, u16),
}

pub enum Focus {
  Tab, Dlg(Task),
}

pub struct App {
  pub params:      SystemParams,
  pub view:        AppView,
  pub focus:       Focus,
  pub request:     Option<Request>,
  pub guide:       String,
  pub clear:       bool,
  pub quit:        bool,
} 


impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {
    let user_text = std::fs::read_to_string(path).unwrap_or_default();
    let params: SystemParams = user_from_str(&user_text).unwrap_or_default();
    let view = AppView::new(
      &Rect::from(Dim(w, h)), 
      &params.style.get_frame_params()
    );
    let mut app = Self {
      guide:       "".into(),
      request:     None,
      focus:       Focus::Tab,
      clear:       true,
      quit:        false,
      view,
      params,
    };

    match url::Url::parse(&app.params.init_url) {
      Err(e) => app.edit_dlg(
        Task::Init(app.params.init_url.clone()), 
        &format!("Try again: {e}"), 
        &app.params.init_url.clone(),
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
    self.guide = format!("Press {} for menu", self.params.keys.menu);
    self.view.dialog = None;
    self.view.push_frame();
  }


  fn ack_dlg(&mut self, prompt: &str) {
    self.focus = Focus::Dlg(Task::Default);
    self.view.dialog(self.params.dlg(&prompt).ack());
  }


  fn ask_dlg(&mut self, task: Task, prompt: &str) {
    self.focus = Focus::Dlg(task);
    self.view.dialog(self.params.dlg(&prompt).ask());
  }


  fn edit_dlg(&mut self, task: Task, prompt: &str, text: &str) {
    self.focus = Focus::Dlg(task);
    self.view.dialog(self.params.dlg(&prompt).edit(text));
  }


  fn select_dlg(&mut self, task: Task, prompt: &str, options: Vec<String>) {
    self.focus = Focus::Dlg(task);
    self.view.dialog(self.params.dlg(&prompt).select(options));
  }


  fn join_gemdoc(&mut self, url: url::Url, response: String, content: String) {
    let Ok(status) = StatusText::try_from(response.as_str()) else {
      self.ack_dlg(
        &format!("Response {response} is not valid for gemini protocol")
      );
      return
    };
    match status.tag {
      Status::InputExpected | 
      Status::InputExpectedSensitive => {
        self.edit_dlg(Task::Reply(url.clone()), &status.text, "");
      }
      Status::RedirectTemporary | 
      Status::RedirectPermanent => match url::Url::parse(&status.text) {
        Err(e)  => self.ack_dlg(&format!("Redirects to invalid URL. {e}")),
        Ok(url) => self.ask_dlg(Task::Go(url.clone()), &status.text),
      }
      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => self.ack_dlg(&status.text),
      _ => {
        self.view.tab(
          &url, 
          PageParams::init().gem_styles(
            gemini::parse_doc(&content),
            |g| self.params.style.get_gem_tag_params(&g.tag)
          )
          .style(&self.params.style.general)
          .draw_point(true)
        );
      }
    }
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
        self.ack_dlg(&e);
        self.request = None;
        true
      }
      Ok((r, c)) => {
        let url = request.url.clone();
        self.join_gemdoc(url, r, c);
        self.request = None;
        true
      }
    }
  }

  pub fn spawn_request(&mut self, url: &url::Url) {
    match (&mut self.request, url.scheme()) {
      (None, "gemini") => self.request = Some(
        Request::new(&url, self.params.timeout)
      ),
      (None, scheme) => self.ack_dlg(
        &format!("Protocol {scheme} not yet supported")
      ),
      (Some(request), _) => {
        let url = request.url.to_string();
        self.ack_dlg(
          &format!("still processing request for {url}")
        );
      }
    }
  }


  pub fn push_style(&mut self) {
  //self.tabs.push_gem_style(
  //  &mut self.view,
  //  &self.params.style.general,
  //  |gem| self.params.style.get_gem_tag_params(gem)
  //);
  //self.view.push_frame();
  }


  pub fn select_link(&mut self, url_str: &str) {
    match self.view.tabs
      .get_current()
      .map(|tab| &tab.url)
      .map(|url| util::join_if_relative(&url, url_str)) 
    {
      None => {},
      Some(Err(e)) => {
        self.ack_dlg(&format!("{url_str} -- Invalid URL. {e}"));
      }
      Some(Ok(url)) => {
        let prompt = &format!("{url} -- Make request?");
        self.ask_dlg(Task::Go(url.into()), prompt);
      } 
    }
  }


  pub fn update(&mut self, message: &Msg) {
    self.clear = false;

    if let Msg::Quit = message {
      self.quit = true;

    } else if let Msg::Resize(w, h) = message {
      self.view.resize(&Rect::from(Dim(*w, *h)));
      self.clear = true;
    } 

    else if 
      let Msg::Action(action) = message &&
      let Focus::Tab = &mut self.focus &&
      let Some(view) = self.view.tabs.get_current_mut()
    {
      match action {
        Action::SaveUrl => 
          if let Some(url) = self.view.tabs
            .get_current()
            .map(|tab| &tab.url)
        {
          match self.params.save_url(url) {
            Err(e) => self.ack_dlg(&e),
            Ok(()) => self.ack_dlg(&format!("Saved URL: {url}")),
          }
        }

        Action::Select => 
          match self.view.tabs
            .get_current()
            .and_then(|tab| tab.page.get_source()) 
        {
          Some(TabText::Gemini(gemtext)) => {
            match &gemtext.tag {
              GemTag::Link(link) => {
                let link = link.clone();
                self.select_link(&link);
              }
              gemtag => self.ack_dlg(
                &format!("You've selected {gemtag:?}")
              )
            }
          }
          _ => self.ack_dlg(
            &format!("You've selected nothing")
          ),
        }

        Action::CycleLeft => {
          let cmd = self.view.tabs.move_wrapped(-1);
          self.view.push_frame();
        }

        Action::CycleRight => {
          let cmd = self.view.tabs.move_wrapped(1);
          self.view.push_frame();
        }

        Action::LoadUrl => self.select_dlg(
          Task::NewTab, "Choose URL: ", self.params.urls.clone(),
        ),

        Action::Menu => self.select_dlg(
          Task::Menu, 
          "Choose: ",
          MENU.iter().map(|s| s.to_string()).collect(),
        ),

        Action::NewTab => self.edit_dlg(
          Task::NewTab, "enter path: ", "",
        ),

        Action::DelTab => self.ask_dlg(
          Task::DelTab, "Delete current tab?",
        ),

        action => {
          action.update(&mut view.page);
        }
      }
    }

    else if 
      let Msg::Action(action) = message &&
      let Focus::Dlg(task) = &mut self.focus &&
      let Some(dlg) = &mut self.view.dialog
    {
      match (task, action, &dlg.dlg_type) {
        (Task::NewTab, Action::Select, DlgType::Select) => {
          if let Some(link) = self.params.urls.get(
            dlg.body.get_index()
          ) {
            let link = link.clone();
            self.select_link(&link);
          } else {
            self.focus_tabs();
          }
        }

        (Task::ChangeKeys, Action::Select, DlgType::Select) => {
          match std::fs::read_to_string(
            user::get_keys_file(&dlg.body.get_param_string())
          ) {
            Err(e) => {
              self.ack_dlg(&format!("Problem: {e}"))
            }
            Ok(s) => if let Err(e) = self.params.keys.update_from_str(&s) {
              self.ack_dlg(&format!("Problem: {e}"));
            } else {
              self.focus_tabs();
            }
          }
        }

        (Task::ChangeStyle, Action::Select, DlgType::Select) => {
          match std::fs::read_to_string(
            user::get_styles_file(&dlg.body.get_param_string())
          ) {
            Err(e) => self.ack_dlg(&e.to_string()),
            Ok(s)  => if let Err(e) = self.params.style.update_from_str(&s) {
              self.ack_dlg(&e.to_string());
              self.push_style();
            } else {
              self.focus_tabs();
              self.push_style();
            }
          }
        }

        (Task::Menu, Action::Select, DlgType::Select) => {
          match MENU[dlg.body.get_index()] {
            MANUAL => {
              self.ack_dlg("View manual".into());
              // write the bloody manual!
            }
            CHANGE_KEYS => match util::get_entries(KEYS_PATH) {
              Err(e)    => self.ack_dlg(&e),
              Ok(entry) => self.select_dlg(
                Task::ChangeKeys, "Choose keys", entry
              ),
            }
            CHANGE_STYLE => match util::get_entries(STYLES_PATH) {
              Err(e)    => self.ack_dlg(&e),
              Ok(entry) => self.select_dlg(
                Task::ChangeStyle, "Choose Style", entry
              ),
            }
            VIEW_SETTINGS => {
              let text = format!("{:#?}", self.params)
                .lines()
                .map(|string| string.into())
                .collect();
              self.select_dlg(Task::Default, "Current Settings", text);
            }
            _ => self.focus_tabs(),
          }
        }

        (Task::Init(_), Action::Enter, _) => {
          let url_str = dlg.body.get_string().unwrap();
          match url::Url::parse(&url_str) {
            Err(e) => self.edit_dlg(
              Task::Init(url_str.clone()),
              &format!("Invalid URL. {e}"),
              &url_str,
            ),
            Ok(url) => {
              self.focus_tabs();
              self.spawn_request(&url);
            }
          }
        }

        (Task::Init(url_str), Action::Cancel, DlgType::Edit) => {
          let url_str = url_str.clone();
          self.ask_dlg(Task::Init(url_str), "Exit application?");
        }

        (Task::Init(url_str), Action::Cancel, _) |
        (Task::Init(url_str), Action::No,     _) => {
          let url_str = url_str.clone();
          self.edit_dlg(
            Task::Init(url_str.clone()), 
            &format!("Enter URL: "),
            &url_str,
          );
        }

        (Task::Reply(url), Action::Enter, _) => {
          let text = dlg.body
            .get_string()
            .unwrap()
            .trim()
            .replace(" ", "%20");
          match url.clone().join(&format!("?{text}")) {
            Err(e) => {
              self.ack_dlg(&format!("Invalid URL. {e}"));
            }
            Ok(url) => {
              self.focus_tabs();
              self.spawn_request(&url);
            }
          }
        }

        (Task::NewTab, Action::Enter, _) => {
          match url::Url::parse(
            &dlg.body.get_string().unwrap()
          ) {
            Err(e) => {
              self.ack_dlg(&format!("Invalid URL: {e}"));
            }
            Ok(url) => {
              self.focus_tabs();
              self.spawn_request(&url);
            }
          }
        }

        (Task::Init(_), Action::Yes, _) => {
          self.quit = true;
        }

        (Task::Go(url), Action::Yes, _) => {
          let url = url.clone();
          self.focus_tabs();
          self.spawn_request(&url);
        }

        (Task::DelTab, Action::Yes, _) => {
          match self.view.tabs.remove() {
            true => {
             // self.layout.apply_remove(TAB, cmd);
              let url_str = self.params.init_url.clone();
              self.edit_dlg(
                Task::Init(url_str.clone()), 
                &format!("Enter URL: "),
                &url_str,
              );
            }
            _ => {
              self.focus_tabs();
            }
          }
        }

        (_,                _, DlgType::Ack) |
        (_,   Action::Select,            _) |
        (_,       Action::No,            _) |
        (_,   Action::Cancel,            _) => {
          self.focus_tabs();
        }

        (_, action, DlgType::Edit) => {
          action.update_edit(&mut dlg.body);
        }

        (_, action, _) => {
          action.update(&mut dlg.body);
        }
      }
    } 
  }


  pub fn get_update(&self, event: crossterm::event::Event) -> Option<Msg> {
    use crossterm::event::{
      Event, KeyEvent, KeyEventKind, KeyModifiers, KeyCode,
    };
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
        Focus::Dlg(_) => 
          if let Some(dlg) = &self.view.dialog {
            self.params.keys
              .get_dlg_action(&dlg.dlg_type, &kc)
              .map(Msg::Action)
          } else {None},
        Focus::Tab => 
          self.params.keys.get_tab_action(&kc).map(Msg::Action),
      }
      _ => None,
    }
  }


  pub fn draw(&self, w: &mut std::io::Stdout) -> std::io::Result<()> {
    use std::io::Write;
    use crossterm::{QueueableCommand, cursor, terminal};
    w.queue(cursor::Hide)?;
    if self.clear {
      w.queue(terminal::Clear(terminal::ClearType::All))?;
    }
    self.view.draw(w)?;
    let banner_text = self.view.tabs.get_banner_text();
    self.view.frame.draw_banner(&banner_text, w)?;
    self.view.frame.draw_footer(&self.guide, w)?;
    w.queue(cursor::Show)?;
    w.flush()
  }
}
