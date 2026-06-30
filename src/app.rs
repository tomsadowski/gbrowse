// src/app.rs

use crate::{
  gemini, 
  util,
  Dim,
  user,
  User, 
  CursorVec,
  Tab,
  TextStyle,
  UserTable,
  user_from_str,
  Request,
  Layout,
  Action,
  Rect, 
  PageViewParams,
  PageParams,
  GemTag, 
  Status, 
  StatusText,
  Frame,
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
  Tab, Dialog(Task),
}

pub struct App {
  pub user:        User,
  pub tabs:        CursorVec<Tab>,
  pub layout:      Layout,
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
    let user: User = user_from_str(&user_text).unwrap_or_default();
    let layout = Layout::from(Rect::from(Dim(w, h)))
      .with_frame_params(user.get_frame_params());
    let mut app = Self {
      guide:       "".into(),
      tabs:        CursorVec::default(),
      request:     None,
      focus:       Focus::Tab,
      new_dlg:     false,
      tab_changed: true,
      clear:       true,
      quit:        false,
      layout,
      user,
    };
    match url::Url::parse(&app.user.init_url) {
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

  fn get_dlg_params(&self, prompt: &[String], text: &[String])
    -> (PageViewParams, PageViewParams) 
  {
    let dlg_1 = PageViewParams::from(
      PageParams::init()
        .with_text(prompt)
        .with_style(self.user.style.info)
    );
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(text)
        .with_style(self.user.style.info)
    ).with_frame_params(self.user.get_dialog_frame_params());
    (dlg_1, dlg_2)
  }

  fn focus_ack_dialog(&mut self, prompt: String) {
    self.guide = format!("Press any key to acknowledge");
    let (dlg_1, dlg_2) = self.get_dlg_params(
      &vec![prompt], &vec![self.guide.clone()]
    );
    self.layout.insert(DLG_1, dlg_1);
    self.layout.insert(DLG_2, dlg_2);
    self.focus = Focus::Dialog(Task::Default);
    self.new_dlg = true;
  }

  fn focus_ask_dialog(&mut self, task: Task, prompt: &str) {
    self.guide = format!(
      "{} yes {} no", self.user.keys.yes, self.user.keys.no
    );
    let (dlg_1, dlg_2) = self.get_dlg_params(
      &vec![prompt.into()], &vec![self.guide.clone()]
    );
    self.layout.insert(DLG_1, dlg_1);
    self.layout.insert(DLG_2, dlg_2);
    self.focus = Focus::Dialog(task);
    self.new_dlg = true;
  }

  fn focus_edit_dialog(&mut self, task: Task, prompt: &str, text: &str) {
    self.guide = format!("Press {} to cancel", self.user.keys.cancel);
    let (dlg_1, dlg_2) = self.get_dlg_params(
      &vec![prompt.into()], &vec![text.into()]
    );
    let dlg_2 = PageViewParams::from(
      PageParams::init()
        .with_text(&vec![text])
        .with_style(self.user.style.info)
        .edit(true)
    )
    .with_frame_params(self.user.get_dialog_frame_params())
    .with_draw_point(true);
    self.layout.insert(DLG_1, dlg_1);
    self.layout.insert(DLG_2, dlg_2);
    self.focus = Focus::Dialog(task);
    self.new_dlg = true;
  }

  fn focus_select_dialog(
    &mut self, 
    task: Task, 
    prompt: &str, 
    options: Vec<String>
  ) {
    let (dlg_1, mut dlg_2) = self.get_dlg_params(
      &vec![prompt.into()], &options
    );
    dlg_2.set_draw_point(true);
    self.layout.insert(DLG_1, dlg_1);
    self.layout.insert(DLG_2, dlg_2);
    self.guide = format!("Press {} to select", self.user.keys.select);
    self.focus = Focus::Dialog(task);
    self.new_dlg = true;
  }

  fn join_gemdoc(&mut self, url: url::Url, response: String, content: String) {
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
      Status::RedirectPermanent => match url::Url::parse(&status.text) {
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
          &mut self.layout,
          &url, 
          gemini::parse_doc(&content), 
          self.user.style.general,
          |g| self.user.get_style_from_gem_text(g),
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
        self.join_gemdoc(url, r, c);
        self.request = None;
        true
      }
    }
  }

  pub fn spawn_request(&mut self, url: &url::Url) {
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

  pub fn push_style(&mut self) {
    self.tabs.push_gem_style(
      &mut self.layout,
      self.user.style.general,
      |gem| self.user.get_style_from_gem_tag(gem)
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
    match (message, &mut self.focus) {
      (Msg::Quit, _) => {
        self.quit = true;
      }
      (Msg::Resize(w, h), _) => {
        self.layout.resize(Rect::from(Dim(*w, *h)));
      }
      (Msg::Action(action), Focus::Dialog(task)) => 
        match (
          self.layout.get_page_view_mut(DLG_2),
          action,
          task
        ) {
        (Some(view), Action::Select, Task::NewTab) => {
          if let Some(link) = self.user.urls.get(
            view.page.get_index()
          ) {
            let link = link.clone();
            self.select_link(&link);
          } else {
            self.focus_tabs();
          }
        }
        (Some(view), Action::Select, Task::ChangeKeys) => {
          match std::fs::read_to_string(
            user::get_keys_file(&view.get_param_string())
          ) {
            Err(e) => self.focus_ack_dialog(format!("Problem: {e}")),
            Ok(s)  => if let Err(e) = self.user.keys.update_from_str(&s) {
              self.focus_ack_dialog(format!("Problem: {e}"));
            } else {
              self.focus_tabs();
            }
          }
        }
        (Some(view), Action::Select, Task::ChangeStyle) => {
          match std::fs::read_to_string(
            user::get_styles_file(&view.get_param_string())
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
        (Some(view), Action::Select, Task::Menu) => {
          match MENU[view.page.get_index()] {
            MANUAL => {
              self.focus_ack_dialog("View manual".into());
            }
            CHANGE_KEYS => match util::get_entries(KEYS_PATH) {
              Err(e)    => self.focus_ack_dialog(e),
              Ok(entry) => self.focus_select_dialog(
                Task::ChangeKeys, "Choose keys", entry
              ),
            }
            CHANGE_STYLE => match util::get_entries(STYLES_PATH) {
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
        (Some(view), Action::Enter, Task::Init(_)) => {
          let url_str = view.get_page_string().unwrap();
          match url::Url::parse(&url_str) {
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
        (Some(editbox), Action::Cancel, Task::Init(url_str)) => {
          let url_str = url_str.clone();
          self.focus_ask_dialog(
            Task::Init(url_str), "Exit application?".into()
          )
        }
        (Some(view), Action::Enter, Task::Reply(url)) => {
          let text = view
            .get_page_string()
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
        (Some(view), Action::Enter, Task::NewTab) => {
          match url::Url::parse(
            &view.get_page_string().unwrap()
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
          if self.tabs.cursor.remove(&mut self.tabs.vec).is_some() {
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
       // (DlgInput::Ack(_), _, _) |
        (_,   Action::Select, _) |
        (_,       Action::No, _) |
        (_,   Action::Cancel, _) => {
          self.focus_tabs();
        }
        (Some(textbox), action, _) => {
         // action.update(textbox);
        }
      //(Some(editbox),   action, _) => {
      // // action.update_edit(editbox);
      //}
        (_, _, _) => {
          self.focus_tabs();
        }
      }
      (Msg::Action(action), Focus::Tab) => match (
        self.layout.get_page_view_mut(TAB),
        action,
      ) {
        (None, _) => {}
        (Some(view), Action::SaveUrl) => 
          if let Some(url) = self.tabs.get_url() {
            match self.user.save_url(url) {
              Err(e) => self.focus_ack_dialog(e),
              Ok(()) => self.focus_ack_dialog(
                format!("Saved URL: {url}")
              ),
            }
          }
        (Some(view), Action::Select) => 
          match self.tabs.get_gem_tag(&view.page) {
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
        (Some(view), Action::CycleLeft) => {
          self.tab_changed = {
            self.tabs.cursor.move_wrapped(&self.tabs.vec, -1); true
          };
        }
        (Some(view), Action::CycleRight) => {
          self.tab_changed = {
            self.tabs.cursor.move_wrapped(&self.tabs.vec, 1); true
          };
        }
        (Some(view), Action::LoadUrl) => {
          self.focus_select_dialog(
            Task::NewTab, 
            "Choose URL: ", 
            self.user.urls.clone()
          );
        }
        (Some(view), Action::Menu) => {
          self.focus_select_dialog(
            Task::Menu, 
            "Choose: ", 
            MENU.iter().map(|s| s.to_string()).collect()
          );
        }
        (Some(view), Action::NewTab) => {
          self.focus_edit_dialog(
            Task::NewTab, "enter path: ", ""
          );
        }
        (Some(view), Action::DelTab) => {
          self.focus_ask_dialog(
            Task::DelTab, "Delete current tab?"
          );
        }
        (Some(view), action) => {
        //self.tabs.use_page_params_mut(
        //  |textbox| action.update(textbox)
        //);
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
        Focus::Dialog(_) => 
          self.user.keys.get_tab_action(&kc).map(Msg::Action),
        Focus::Tab => 
          self.user.keys.get_tab_action(&kc).map(Msg::Action),
      }
      _ => None,
    }
  }

  pub fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, cursor, terminal};
    w.queue(cursor::Hide)?;
    if self.clear {
      w.queue(terminal::Clear(terminal::ClearType::All))?;
      self.layout.frame.draw(w)?;
    }
    let banner_text = self.tabs.get_banner_text();
    self.layout.frame.draw_banner(&banner_text, w)?;
    self.layout.frame.draw_footer(&self.guide, w)?;
    self.layout.draw(w)?;
    w.flush()
 // if let Focus::Dialog(_) = &self.focus {
 //   //dialog.draw(w)?;
 // } else {
 //   if let Some(request) = &self.request {
 // //  let tb: TextBox = TextBox::from(
 // //      self.frame.get_view_port().top_row()
 // //    ).reference(
 // //      &vec![format!("requesting {}", request.url)],
 // //      |s| StyledText::from(s.clone())
 // //    );
 // //  tb.draw(writer)?;
 //     self.tabs.draw(w)?;
 //   } else {
 //     self.tabs.draw(w)?;
 //   }
 // }
  }
}
