// src/app.rs

use crate::{
    gemini, 
    Status,
    Dialog,
    util,
    TabText,
    Dim,
    UserConfig, 
    Draw,
    AppView,
    GemText,
    DialogType,
    DialogParams,
    UserTable,
    Request,
    Action,
    Rect, 
    PageParams,
    GemTag, 
    StatusText,
    Resize,
};



#[derive(Debug)]
pub enum Task {
    Default, 
    NewTab,
    DelTab,
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
    pub config:      UserConfig,
    pub view:        AppView,
    pub focus:       Focus,
    pub request:     Option<Request>,
    pub guide:       String,
    pub clear:       bool,
    pub quit:        bool,
} 

impl App {
    pub fn init(path: &str, w: u16, h: u16) -> Self {
        let config_str = std::fs::read_to_string(path).unwrap_or_default();
        let (config, config_result) = UserConfig::from_str(&config_str, &());
        let view = AppView::new(
            &Rect::from(Dim(w, h)), 
            &config.style.get_frame_params()
        );
        let mut app = Self {
            guide:       "".into(),
            request:     None,
            focus:       Focus::Tab,
            clear:       true,
            quit:        false,
            view,
            config,
        };
        match (url::Url::parse(&app.config.init_url), config_result) {
            (Err(e), Err(msg)) => {
                app.edit_dlg(
                    Task::Init(app.config.init_url.clone()), 
                    &format!("
                        Config issue: {msg},
                        {e}, enter revised URL:
                    "), 
                    &app.config.init_url.clone(),
                );
            }
            (Err(e), Ok(())) => {
                app.edit_dlg(
                    Task::Init(app.config.init_url.clone()), 
                    &format!("
                        {e}, enter revised URL:
                    "), 
                    &app.config.init_url.clone(),
                );
            }
            (Ok(url), Err(msg)) => {
                app.ack_dlg(&format!("Config issue: {msg}"));
                app.spawn_request(&url);
            }
            (Ok(url), Ok(())) => {
                app.focus_tabs();
                app.spawn_request(&url);
            }
        }
        app
    }

    pub fn push_style(&mut self) {
        self.view.update_frame_params(&self.config.style.get_frame_params());
        for tab in self.view.tabs.data.iter_mut() {
            tab.page.restyle(
                |text| self.config.style.get_tab_text_params(text)
            );
            tab.page.style = self.config.style.general.style;
        }
        self.view.push_frame();
        self.clear = true;
    }

    pub fn focus_tabs(&mut self) {
        self.focus = Focus::Tab;
        self.guide = format!("Press {} for menu", self.config.keys.menu);
        self.view.dialog = None;
        self.view.reset_frame();
    }

    fn flash(&mut self, prompt: &str) {
        self.view.flash(
            DialogParams::from(&self.config)
                .prompt(&prompt)
        );
    }

    fn ack_dlg(&mut self, prompt: &str) {
        self.focus = Focus::Dlg(Task::Default);
        self.view.dialog(
            DialogParams::from(&self.config)
                .prompt(&prompt)
                .ack()
        );
    }

    fn ask_dlg(&mut self, task: Task, prompt: &str) {
        self.focus = Focus::Dlg(task);
        self.view.dialog(
            DialogParams::from(&self.config)
                .prompt(&prompt)
                .ask()
        );
    }

    fn edit_dlg(&mut self, task: Task, prompt: &str, text: &str) {
        self.focus = Focus::Dlg(task);
        self.view.dialog(
            DialogParams::from(&self.config)
                .prompt(&prompt)
                .edit(text)
        );
    }

    fn select_dlg(&mut self, task: Task, prompt: &str, options: Vec<String>) {
        self.focus = Focus::Dlg(task);
        self.view.dialog(
            DialogParams::from(&self.config)
                .prompt(&prompt)
                .select(options)
        );
    }

    fn join_gemdoc(&mut self, url: url::Url, response: String, content: String) {
        match StatusText::try_from(response.as_str()) {
            Ok(StatusText {tag, text}) => match tag {
                Status::InputExpected | 
                Status::InputExpectedSensitive => {
                    self.edit_dlg(Task::Reply(url.clone()), &text, "");
                }
                Status::RedirectTemporary | 
                Status::RedirectPermanent => {
                    match url::Url::parse(&text) {
                        Err(e) => self.ack_dlg(
                            &format!("Redirects to invalid URL. {e}")
                        ),
                        Ok(url) => self.ask_dlg(
                            Task::Go(url.clone()), &text
                        ),
                    }
                }
                Status::CertRequiredClient | 
                Status::CertRequiredTransient | 
                Status::CertRequiredAuthorized => {
                    self.ack_dlg(&text);
                }
                _ => {
                    self.view.tab(
                        &url, 
                        PageParams::init()
                            .style(&self.config.style.general)
                            .text_styles(
                                gemini::parse_doc(&content)
                                    .into_iter()
                                    .map(TabText::Gemini)
                                    .collect(),
                                |g| self.config.style.get_tab_text_params(g)
                            )
                    );
                }
            }
            Err(e) => {
                if 0 == self.view.tabs.data.len() {
                    self.edit_dlg(
                        Task::Init(self.config.init_url.clone()), 
                        &format!("Invalid response for Gemini: {e}."),
                        &self.config.init_url.clone(),
                    );
                } else {
                    self.ack_dlg(
                        &format!("Invalid response for Gemini: {e}."),
                    );
                }
            }
        }
    }

    pub fn join_request(&mut self) -> bool {
        // there is no request to join
        let Some(request) = &mut self.request else {
            return false
        };
        // there is a request but it's not ready
        if !request.handle.is_finished() {
            return false
        }
        self.view.flash = None;
        match request.rx.recv() {
            Err(e) => {
                self.ack_dlg(&format!("{e}"));
                self.view.reset_frame();
            }
            Ok(Err(e)) => {
                if 0 == self.view.tabs.data.len() {
                    self.edit_dlg(
                        Task::Init(self.config.init_url.clone()), 
                        &format!("{e}"),
                        &self.config.init_url.clone(),
                    );
                } else {
                    self.ack_dlg(&format!("{e}"));
                }
                self.view.reset_frame();
            }
            Ok(Ok((r, c))) => {
                // assume only gemdoc is supported for now
                let url = request.url.clone();
                self.join_gemdoc(url, r, c);
            }
        }
        self.request = None;
        true
    }

    // Checks the scheme of the given URL against supported schemes.
    // An error dialog displays if not supported.
    // Otherwise, a new request is spawned (but ideally, queued).
    pub fn spawn_request(&mut self, url: &url::Url) {
        match (&mut self.request, url.scheme()) {
            // queue not yet implemented
            (Some(request), _) => {
                let url = request.url.to_string();
                self.ack_dlg(&format!("
                    still processing request for {url}
                "));
            }
            (None, "gemini") => {
                self.request = Some(Request::new(&url, self.config.timeout));
                self.flash(&format!("pending request: {url}"));
            }
            (None, scheme) => {
                if 0 == self.view.tabs.data.len() {
                    self.edit_dlg(
                        Task::Init(self.config.init_url.clone()), 
                        &format!("Protocol: {scheme} not yet supported."),
                        &self.config.init_url.clone(),
                    );
                } else {
                    self.ack_dlg(
                        &format!("Protocol {scheme} not yet supported")
                    );
                }
            }
        }
    }

    pub fn select_link(&mut self, url_str: &str) {
        match self.view.tabs
            .get()
            .map(|tab| &tab.url)
            .map(|url| util::join_if_relative(&url, url_str)) 
        {
            None => {},
            Some(Err(e)) => {
                self.ack_dlg(&format!("{url_str} -- Invalid URL. {e}"));
            }
            Some(Ok(url)) => {
                let prompt = &format!("{url} --\nMake request?");
                self.ask_dlg(Task::Go(url.into()), prompt);
            } 
        }
    }

    pub fn update(&mut self, message: &Msg) {
        self.clear = false;
        self.view.reset_draw_state();

        if let Msg::Quit = message {
            self.quit = true;
        } else 
            if let Msg::Resize(w, h) = message {
            self.view.resize(&Rect::from(Dim(*w, *h)));
            self.clear = true;
        } else 
            if let Msg::Action(action) = message
            && let Focus::Tab = &mut self.focus
            && let Some(tab) = self.view.tabs.get_mut()
        {
            match action {
                Action::SaveUrl => {
                    let url = tab.url.clone();
                    match self.config.save_url(&url) {
                        Err(e) => self.ack_dlg(&e),
                        Ok(()) => self.ack_dlg(&format!("Saved URL: {url}")),
                    }
                }
                Action::Select if let Some(source) = tab.page.get_source() => {
                    match source {
                        TabText::Gemini(
                            GemText {tag: GemTag::Link(link), ..}
                        ) => {
                            let link = link.clone();
                            self.select_link(&link);
                        }
                        TabText::Gemini(GemText {tag, ..}) => {
                            let msg = format!("You've selected {tag:?}");
                            self.ack_dlg(&msg);
                        }
                        _ => self.ack_dlg(&format!("You've selected nothing")),
                    }
                }
                Action::CycleLeft => {
                    self.view.tabs.move_wrapped(-1);
                    self.view.reset_frame();
                }
                Action::CycleRight => {
                    self.view.tabs.move_wrapped(1);
                    self.view.reset_frame();
                }
                Action::LoadUrl => self.select_dlg(
                    Task::NewTab, "Choose URL: ", self.config.urls.clone(),
                ),
                Action::Menu => self.select_dlg(
                    Task::Menu, 
                    "Choose: ",
                    util::MENU.iter().map(|s| s.to_string()).collect(),
                ),
                Action::NewTab => self.edit_dlg(
                    Task::NewTab, "enter path: ", "",
                ),
                Action::DelTab => self.ask_dlg(
                    Task::DelTab, "Delete current tab?",
                ),
                action => {
                    action.update(&mut tab.page);
                }
            }
        } else 
        if let Msg::Action(action) = message
        && let Focus::Dlg(task) = &mut self.focus
        && let Some(Dialog {body: Some(body), dlg_type, ..}) 
            = &mut self.view.dialog 
        {
            match (task, action, dlg_type) {
                (Task::NewTab, Action::Select, DialogType::Select) => {
                    if let Some(link) = self.config.urls.get(body.get_index()) {
                        let link = link.clone();
                        self.select_link(&link);
                    } else {
                        self.focus_tabs();
                    }
                }
                (Task::ChangeKeys, Action::Select, DialogType::Select) => {
                    match std::fs::read_to_string(
                        util::get_keys_file(&body.get_param_string())
                    ) {
                        Err(e) => {
                            self.ack_dlg(&format!("Problem: {e}"))
                        }
                        Ok(s) => {
                            if let Err(e) = self.config.keys
                                .update_from_str(&s, &()) 
                            {
                                self.ack_dlg(&format!("Problem: {e}"));
                            } else {
                                self.focus_tabs();
                            }
                        }
                    }
                }
                (Task::ChangeStyle, Action::Select, DialogType::Select) => {
                    match std::fs::read_to_string(
                        util::get_styles_file(&body.get_param_string())
                    ) {
                        Err(e) => {
                            self.ack_dlg(&e.to_string());
                        }
                        Ok(s) => {
                            if let Err(e) = self.config.style
                                .update_from_str(&s, &()) 
                            {
                                self.ack_dlg(&e.to_string());
                            } else {
                                self.focus_tabs();
                            }
                            self.push_style();
                        }
                    }
                }
                (Task::Menu, Action::Select, DialogType::Select) => {
                    match util::MENU[body.get_index()] {
                        util::MANUAL => {
                            self.ack_dlg("
                                Manual not yet written, but I will get to it.
                            ".into());
                        }
                        util::CHANGE_KEYS => 
                            match util::get_entries(util::KEYS_PATH) 
                        {
                            Err(e) => self.ack_dlg(&e),
                            Ok(entry) => self.select_dlg(
                                Task::ChangeKeys, "Choose keys", entry
                            ),
                        }
                        util::CHANGE_STYLE => 
                            match util::get_entries(util::STYLES_PATH) 
                        {
                            Err(e) => self.ack_dlg(&e),
                            Ok(entry) => self.select_dlg(
                                Task::ChangeStyle, "Choose Style", entry
                            ),
                        }
                        util::VIEW_SETTINGS => {
                            let text = format!("{:#?}", self.config)
                                .lines()
                                .map(|str| str.into())
                                .collect();
                            self.select_dlg(
                                Task::Default, "Current Settings", text
                            );
                        }
                        _ => self.focus_tabs(),
                    }
                }
                (Task::Init(_), Action::Enter, _) => {
                    let url_str = body.get_string().unwrap();
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
                (Task::Init(url_str), Action::Cancel, DialogType::Edit) => {
                    let url_str = url_str.clone();
                    self.ask_dlg(Task::Init(url_str), "Exit application?");
                }
                (Task::Init(url_str), Action::Cancel, _) |
                (Task::Init(url_str), Action::No, _) => {
                    let url_str = url_str.clone();
                    self.edit_dlg(
                        Task::Init(url_str.clone()), 
                        &format!("Enter URL: "),
                        &url_str,
                    );
                }
                (Task::Reply(url), Action::Enter, _) => {
                    let text = body
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
                    match url::Url::parse(&body.get_string().unwrap()) {
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
                    self.view.tabs.remove(); 
                    if 0 == self.view.tabs.data.len() {
                        let url_str = self.config.init_url.clone();
                        self.edit_dlg(
                            Task::Init(url_str.clone()), 
                            &format!("Enter URL: "),
                            &url_str,
                        );
                    } else {
                        self.focus_tabs();
                    }
                }
                (_, _, DialogType::Ack) |
                (_, Action::Select, _) |
                (_, Action::No, _) |
                (_, Action::Cancel, _) => {
                    self.focus_tabs();
                }
                (_, action, DialogType::Edit) => {
                    action.update_edit(body);
                }
                (_, action, _) => {
                    action.update(body);
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
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL, 
                code: KeyCode::Char('c'), ..
            }) => {
                Some(Msg::Quit)
            }
            Event::Key(KeyEvent {
                code, kind: KeyEventKind::Press, ..
            }) => 
                if let Focus::Tab = &self.focus {
                    self.config.keys
                        .get_tab_action(&code)
                        .map(Msg::Action)
                } else if let Some(dlg) = &self.view.dialog {
                    self.config.keys
                        .get_dlg_action(&dlg.dlg_type, &code)
                        .map(Msg::Action)
                }
                else {None}
            _ => None,
        }
    }

    pub fn draw(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        use crossterm::{QueueableCommand, cursor, terminal};

        w.queue(cursor::Hide)?;
        if self.clear {
            w.queue(terminal::Clear(terminal::ClearType::All))?;
        }

        self.view.draw(w)?;
        let banner_text = self.view.tabs.get_banner_text();
        self.view.frame.draw_banner(&banner_text, w)?;
        self.view.frame.draw_footer(&self.guide, w)?;

        if let Focus::Tab = self.focus
        && let Some(page) = self.view.tabs.get().map(|t| &t.page) {
            page.point_view.draw(w)?;
        } 
        else 
        if let Some(Dialog {body: Some(body), dlg_type, ..}) 
            = &self.view.dialog 
        && let DialogType::Select | DialogType::Edit = dlg_type
        {
            body.point_view.draw(w)?;
        }
        w.flush()
    }
}
