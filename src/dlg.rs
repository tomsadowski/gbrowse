// src/dlg.rs

use crate::{
    UserConfig, 
    FrameParams,
    Frame,
    PageParams,
    Page,
    View,
    Rect,
    resize_views,
    BuildView,
};
use std::fmt::Debug;
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub enum DialogType {
    Ack, 
    Ask, 
    Edit, 
    Select,
    Flash,
}

pub struct DialogParams<'a> {
    params:      &'a UserConfig, 
    frame:       FrameParams,
    dialog_type: DialogType,
    prompt:      Option<PageParams<String>>,
    body:        Option<PageParams<String>>,
}
impl<'a> From<&'a UserConfig> for DialogParams<'a> {
    fn from(params: &'a UserConfig) -> Self {
        Self {
            frame: params.style.get_dialog_frame_params(),
            prompt: None,
            body: None,
            dialog_type: DialogType::Flash,
            params
        }
    }
}
impl<'a> DialogParams<'a> { 
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = Some(
            PageParams::new()
                .text(vec![prompt.to_string()])
                .style(&self.params.style.dialog_prompt)
                .max(Some(2))
        );
        self
    }

    pub fn ack(mut self) -> Self {
        if let Some(prompt) = self.prompt {
            self.prompt = Some(prompt.style(&self.params.style.dialog_body).max(None));
        }
        self.body = Some(
            PageParams::new()
                .text(vec![format!("Press any key to acknowledge")])
                .style(&self.params.style.dialog_prompt)
                .max(Some(2))
        );
        self.dialog_type = DialogType::Ack;
        self
    }

    pub fn ask(mut self) -> Self {
        let guide = format!(
            "{} yes {} no", self.params.keys.yes, self.params.keys.no
        );
        if let Some(prompt) = self.prompt {
            self.prompt = Some(prompt.style(&self.params.style.dialog_body).max(None));
        }
        self.body = Some(
            PageParams::new()
                .text(vec![guide])
                .style(&self.params.style.dialog_prompt)
                .max(Some(2))
        );
        self.dialog_type = DialogType::Ask;
        self
    }

    pub fn edit(mut self, text: &str) -> Self {
        self.body = Some(
            PageParams::new()
                .text(vec![text.to_string()])
                .style(&self.params.style.dialog_body)
                .edit(true)
        );
        self.dialog_type = DialogType::Edit;
        self
    }

    pub fn select(mut self, options: Vec<String>) -> Self {
        self.body = Some(
            PageParams::new()
                .text(options)
                .style(&self.params.style.dialog_body)
        );
        self.dialog_type = DialogType::Select;
        self
    }
}
impl<'a> BuildView<Dialog> for DialogParams<'a> {
    fn build(self, rect: &Rect) -> Dialog {
        let frame = self.frame.build_from_outer(rect);
        let mut rect = rect.clone();
        let mut views: Vec<_> = vec![];
        let mut params = vec![];
        if let Some(page) = self.prompt {
            params.push(page);
        }
        if let Some(page) = self.body {
            params.push(page);
        }
        for param in params.into_iter() {
            let v = param.build(&rect);
            let vheight = v.get_display_height().min(rect.h());
            rect = rect.shift_north((vheight as i16) * -1);
            views.push(v);
        }
        let mut inner = frame.inner_rect.clone();
        inner.h = views.iter().map(|v| v.get_display_height()).sum();
        let frame = self.frame.build_from_inner(&inner);
        Dialog {
            frame, 
            dlg_type: self.dialog_type.clone(), 
            body: views.pop(),
            prompt: views.pop(),
        }
    }
}

pub struct Dialog {
    pub frame:      Frame,
    pub dlg_type:   DialogType,
    pub prompt:     Option<Page<String>>,
    pub body:       Option<Page<String>>,
}
impl View for Dialog {
    fn get_max_height(&self) -> u16 {
        self.prompt.as_ref().map(|p| p.get_max_height()).unwrap_or(0)
            + self.body.as_ref().map(|p| p.get_max_height()).unwrap_or(0)
            + (self.frame.screen.h - self.frame.inner_rect.h)
    }

    fn get_display_height(&self) -> u16 {
        self.frame.get_display_height()
    }

    fn draw(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.frame.draw(w)?;
        if let Some(page) = &self.prompt {
            page.draw(w)?;
        }
        if let Some(page) = &self.body {
            page.draw(w)?;
        }
        Ok(())
    }

    fn resize(&mut self, rect: &Rect) {
        let frame = self.frame.params.build_from_outer(rect);
        resize_views(
            &frame.inner_rect, 
            &mut self.get_view_list_mut()
        );
        let mut inner_rect = frame.inner_rect.clone();
        inner_rect.h = 
            self.prompt.as_ref().map(|p| p.get_display_height()).unwrap_or(0)
            + self.body.as_ref().map(|p| p.get_display_height()).unwrap_or(0);
        self.frame = self.frame.params.build_from_inner(&inner_rect);
    }
}
impl Dialog {
    pub fn get_view_list_mut<'a>(&'a mut self) -> Vec<&'a mut dyn View> {
        let mut v = vec![];
        if let Some(page) = self.prompt.as_mut() {
            v.push(page as &mut dyn View);
        }
        if let Some(page) = self.body.as_mut() {
            v.push(page as &mut dyn View);
        }
        v
    }

    pub fn get_debug_hashmap(&self) -> HashMap<&'static str, Box<dyn Debug>> {
        let mut map = HashMap::new();
        map.insert("max", Box::new(self.dlg_type.clone()) as Box<dyn Debug>);
        map.insert("max_height", Box::new(self.get_max_height()) as Box<dyn Debug>);
        map.insert("display_height", Box::new(self.get_display_height()) as Box<dyn Debug>);
        map.insert("frame", Box::new(self.frame.get_debug_hashmap()) as Box<dyn Debug>);
        if let Some(page) = &self.prompt {
            map.insert("prompt", Box::new(page.get_debug_hashmap()) as Box<dyn Debug>);
        }
        if let Some(page) = &self.body {
            map.insert("body", Box::new(page.get_debug_hashmap()) as Box<dyn Debug>);
        }
        map
    }
}
