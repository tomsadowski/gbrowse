// src/color.rs

use crossterm::style::Color;



#[derive(Copy, Clone, Debug, Default)]
pub struct Style {
    pub underline: bool,
    pub bold: bool,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl crossterm::Command for Style {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let mut contentstyle = crossterm::style::ContentStyle::new();
        contentstyle.foreground_color = self.fg;
        contentstyle.background_color = self.bg;
        let mut attributes = crossterm::style::Attributes::none();
        if self.bold {
            attributes.set(crossterm::style::Attribute::Bold);
        }
        if self.underline {
            attributes.set(crossterm::style::Attribute::Underlined);
        }
        contentstyle.attributes = attributes;
        crossterm::style::SetStyle(contentstyle).write_ansi(f)?;
        Ok(())
    }
}
