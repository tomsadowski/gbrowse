// src/view.rs

use crate::{
    PageParams,
    FrameParams,
    Rect,
    Frame,
    DialecticalResize,
    View,
    fill,
    resize_views,
    GetDisplayHeight,
    Dialog,
    DialogParams,
    CursorVec,
    BuildView,
    Draw,
    Resize,
    GetMaxHeight,
    GemText,
    Page,
};
use url::Url;
use std::collections::HashMap;



pub struct AppView {
    pub draw_frame: bool,
    pub rect:       Rect,
    pub frame:      Frame,
    pub flash:      Option<Dialog>,
    pub dialog:     Option<Dialog>,
    pub tabs:       CursorVec<Tab>,
}

impl crate::Draw for AppView {
    fn draw(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        if self.draw_frame {
            self.frame.draw_east(w)?;
            self.frame.draw_west(w)?;
        }
        for view in self.get_view_list() {
            view.draw(w)?;
        }
        fill(&self.rect, &self.frame, w)?;
        Ok(())
    }
}

impl crate::Resize for AppView {
    fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.frame = self.frame.params.build_from_outer(rect);
        self.push_frame();
    }
}

impl AppView {
    pub fn new(rect: &Rect, params: &FrameParams) -> Self {
        let mut appview = Self {
            draw_frame: true,
            frame: params.build_from_outer(rect),
            flash: None,
            dialog: None,
            tabs: CursorVec::default(),
            rect: rect.clone(),
        };
        appview.push_frame();
        appview
    }

    pub fn update_frame_params(&mut self, params: &FrameParams) {
        self.frame = params.build_from_outer(&self.rect);
    }

    pub fn reset_draw_state(&mut self) {
        self.draw_frame = false;
    }

    pub fn reset_frame(&mut self) {
        self.frame = self.frame.params.build_from_outer(&self.rect);
        self.push_frame();
    }

    pub fn push_frame(&mut self) {
        self.draw_frame = true;
        let mut inner_rect = self.frame.inner_rect;
        resize_views(
            &inner_rect, 
            &mut self.get_view_list_mut().iter_mut().collect()
        );
        inner_rect.h = self
            .get_view_list()
            .iter()
            .map(|v| v.get_display_height())
            .sum();
        self.frame = self.frame.params.build_from_inner(&inner_rect);
        log::debug!("\n{:#?}", self.get_debug_hashmap());
    }
    
    pub fn flash(&mut self, params: DialogParams) {
        self.frame = self.frame.params.build_from_outer(&self.rect);
        self.flash = Some(params.build(&self.frame.inner_rect));
        self.push_frame();
    }

    pub fn dialog(&mut self, params: DialogParams) {
        self.frame = self.frame.params.build_from_outer(&self.rect);
        self.dialog = Some(params.build(&self.frame.inner_rect));
        self.push_frame();
    }

    pub fn tab(&mut self, url: &url::Url, params: PageParams<TabText>) {
        self.frame = self.frame.params.build_from_outer(&self.rect);
        self.tabs.insert_unique_with(
            |tab| &tab.url == url,
            Tab {
                url: url.clone(), 
                page: params.build(&self.frame.inner_rect)
            },
        );
        self.push_frame();
    }

    pub fn get_view_list(&self) -> Vec<Option<&dyn View>> {
        vec![
            self.flash.as_ref().map(|x| x as &dyn View),
            self.dialog.as_ref().map(|x| x as &dyn View),
            self.tabs.get().map(|f| &f.page as &dyn View),
        ]
    }

    pub fn get_view_list_mut<'a>(&'a mut self) -> Vec<Option<&'a mut dyn View>> {
        vec![
            self.flash.as_mut().map(|x| x as &mut dyn View),
            self.dialog.as_mut().map(|x| x as &mut dyn View),
            self.tabs.get_mut().map(|f| &mut f.page as &mut dyn View),
        ]
    }

    pub fn get_debug_hashmap(&self) -> HashMap<&'static str, Box<dyn std::fmt::Debug>> {
        use std::fmt::Debug;
        HashMap::from([ 
            ("screen", Box::new(format!("{:?}", self.rect)) as Box<dyn Debug>),
            ("frame", Box::new(self.frame.get_debug_hashmap()) as Box<dyn Debug>),
            ("flash", Box::new(
                    self.flash.as_ref()
                        .map(|dlg| dlg.get_debug_hashmap()).unwrap_or_default()
                ) as Box<dyn Debug>
            ),
            ("dialog", Box::new(
                    self.dialog.as_ref()
                        .map(|dlg| dlg.get_debug_hashmap()).unwrap_or_default()
                ) as Box<dyn Debug>
            ),
            ("tab", Box::new(
                    self.tabs.get().as_ref()
                        .map(|tab| tab.page.get_debug_hashmap()).unwrap_or_default()
                ) as Box<dyn Debug>
            ),
        ])
    }
}


pub enum TabText {
    Gemini(GemText),
    Gopher(String),
}

impl std::fmt::Display for TabText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Gemini(gemtext) => gemtext.fmt(f),
            Self::Gopher(string) => string.fmt(f),
        }
    }
}


pub struct Tab {
    pub url: Url,
    pub page: Page<TabText>,
} 

impl GetMaxHeight for CursorVec<Tab> {
    fn get_max_height(&self) -> u16 {
        if let Some(view) = self.get() {
            view.page.get_max_height()
        } else {
            u16::MAX
        }
    }
}

impl Draw for CursorVec<Tab> {
    fn draw(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        if let Some(view) = self.get() {
            view.page.draw(writer)?;
        }
        Ok(())
    }
}

impl Resize for CursorVec<Tab> {
    fn resize(&mut self, rect: &Rect) {
        for tab in self.data.iter_mut() {
            tab.page.resize(rect);
        }
    }
}

impl CursorVec<Tab> {
    pub fn get_banner_text(&self) -> String {
        match self.get().map(|tab| tab.url.to_string()) {
            None => format!("Empty"),
            Some(s) => format!(
                "{}/{} - {s}", self.cursor.head + 1, self.data.len()
            ),
        }
    }
}
