
// src/userstyle.rs

use crate::{
    UserAssign, 
    UserTable,
    MarginParams,
    BorderParams,
    TextParams,
    TabText,
    GemTag,
    GemText,
    FrameParams,
    Style,
    color,
    util,
};
use toml::{Value, map::Map};



#[derive(Debug)]
pub enum StyleConfigField {
    Palette,
    Border(BorderField), 
    Margin(StyleMarginField), 
    Text(StyleTextField),
}

#[derive(Debug)]
pub enum BorderField {
    App, 
    Dialog,
}

#[derive(Debug)]
pub enum StyleMarginField {
    Text, 
    Screen,
    DialogText,
    DialogScreen,
}

#[derive(Debug)]
pub enum StyleTextField {
    General,
    Banner,
    Footer,
    DialogBody,
    DialogHeading,
    Text,
    Heading3,
    Heading2,
    Heading1,
    Preformat,
    Link,
    Error,
    Quote,
    List,
}

impl std::str::FromStr for StyleConfigField {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "palette"               => Ok(Self::Palette),
            "border"                => Ok(Self::Border(BorderField::App)),
            "dialog_border"         => Ok(Self::Border(BorderField::Dialog)),
            "text_margin"           => Ok(Self::Margin(StyleMarginField::Text)),
            "screen_margin"         => Ok(Self::Margin(StyleMarginField::Screen)),
            "dialog_text_margin"    => Ok(Self::Margin(StyleMarginField::DialogText)),
            "dialog_screen_margin"  => Ok(Self::Margin(StyleMarginField::DialogScreen)),
            "general"               => Ok(Self::Text(StyleTextField::General)),
            "banner"                => Ok(Self::Text(StyleTextField::Banner)),
            "footer"                => Ok(Self::Text(StyleTextField::Footer)),
            "dialog_body"           => Ok(Self::Text(StyleTextField::DialogBody)),
            "dialog_heading"        => Ok(Self::Text(StyleTextField::DialogHeading)),
            "text"                  => Ok(Self::Text(StyleTextField::Text)),
            "heading3"| "h3"        => Ok(Self::Text(StyleTextField::Heading3)),
            "heading2" | "h2"       => Ok(Self::Text(StyleTextField::Heading2)),
            "heading1" | "h1"       => Ok(Self::Text(StyleTextField::Heading1)),
            "preformat"             => Ok(Self::Text(StyleTextField::Preformat)),
            "link"                  => Ok(Self::Text(StyleTextField::Link)),
            "error"                 => Ok(Self::Text(StyleTextField::Error)),
            "quote"                 => Ok(Self::Text(StyleTextField::Quote)),
            "list"                  => Ok(Self::Text(StyleTextField::List)),
            string => Err(format!("
                Style table does not contain field {string}
            ")),
        }
    }
}

#[derive(Debug)]
pub enum MarginParamsField {
    North, 
    South, 
    East, 
    West
}

impl std::str::FromStr for MarginParamsField {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "north" | "n"   => Ok(Self::North),
            "south" | "s"   => Ok(Self::South),
            "east" | "e"    => Ok(Self::East),
            "west" | "w"    => Ok(Self::West),
            string => Err(format!("
                Margin table does not contain field {string}
            ")),
        }
    }
}

#[derive(Debug)]
pub enum TextStyleParamsField {
    Wrap, 
    Style(StyleField)
}

impl std::str::FromStr for TextStyleParamsField {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "wrap" => Ok(Self::Wrap),
            string => StyleField::from_str(string).map(|s| Self::Style(s)),
        }
    }
}

#[derive(Debug)]
pub enum BorderParamsField {
    Style(StyleField), 
    Corner, 
    Bracket,
}

impl std::str::FromStr for BorderParamsField {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "corner"    => Ok(Self::Corner),
            "bracket"   => Ok(Self::Bracket),
            string => StyleField::from_str(string).map(|s| Self::Style(s))
        }
    }
}

#[derive(Debug)]
pub enum StyleField {
    Color(ColorField), 
    Attribute(AttributeField)
}

#[derive(Debug)]
pub enum ColorField {
    Fg, 
    Bg
}

#[derive(Debug)]
pub enum AttributeField {
    Bold, 
    Underline
}

impl std::str::FromStr for StyleField {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "fg"        => Ok(Self::Color(ColorField::Fg)),
            "bg"        => Ok(Self::Color(ColorField::Bg)),
            "bold"      => Ok(Self::Attribute(AttributeField::Bold)),
            "underline" => Ok(Self::Attribute(AttributeField::Underline)),
            string => Err(format!("
                Style table does not contain field {string}
            ")),
        }
    }
}


#[derive(Clone, Default, Debug)]
pub struct StyleConfig {
    pub palette: Map<String, Value>,
    pub text_margin: MarginParams,
    pub screen_margin: MarginParams,
    pub border: Option<BorderParams>,
    pub general: TextParams,
    pub banner: TextParams,
    pub footer: TextParams,
    pub dialog_body: TextParams,
    pub dialog_prompt: TextParams,
    pub dialog_border: BorderParams,
    pub dialog_text_margin: MarginParams,
    pub dialog_screen_margin: MarginParams,
    pub text: TextParams,
    pub heading3: TextParams,
    pub heading2: TextParams,
    pub heading1: TextParams,
    pub preformat: TextParams,
    pub link: TextParams,
    pub error: TextParams,
    pub quote: TextParams,
    pub list: TextParams,
} 

impl StyleConfig {
    pub fn get_frame_params(&self) -> FrameParams {
        FrameParams::init()
            .screen_margin(self.screen_margin)
            .text_margin(self.text_margin)
            .banner_style(&self.banner)
            .footer_style(&self.footer)
            .margin_style(&self.general)
            .border_style(self.border)
    }

    pub fn get_dialog_frame_params(&self) -> FrameParams {
        FrameParams::init()
            .screen_margin(self.dialog_screen_margin)
            .text_margin(self.dialog_text_margin)
            .margin_style(&self.dialog_body)
            .border_style(Some(self.dialog_border))
    }

    pub fn get_tab_text_params(&self, text: &TabText) -> TextParams {
        match text {
            TabText::Gemini(gemtext) => self.get_gem_text_params(gemtext),
            _ => TextParams::default(),
        }
    }

    pub fn get_gem_text_params(&self, text: &GemText) -> TextParams {
        self.get_gem_tag_params(&text.tag)
    }

    pub fn get_gem_tag_params(&self, tag: &GemTag) -> TextParams {
        match tag {
            GemTag::HeadingOne   => self.heading1.into(),
            GemTag::HeadingTwo   => self.heading2.into(),
            GemTag::HeadingThree => self.heading3.into(),
            GemTag::Text         => self.text.into(),
            GemTag::PreFormat    => self.preformat.into(),
            GemTag::Link(_)      => self.link.into(),
            GemTag::ListItem     => self.list.into(),
            GemTag::Quote        => self.quote.into(),
        }
    }
}

impl UserAssign<()> for StyleConfig {
    type Field = StyleConfigField;

    fn load_context(&mut self, table: &mut toml::Table) {
        if let Some(Value::Table(palette)) = table.remove("palette") {
            self.palette = palette;
        }
    }

    fn assign(
        &mut self, field: Self::Field, value: Value, _: &()
    ) -> Result<(), String> {

        match (field, value) {
            (StyleConfigField::Border(field), Value::Table(value)) => {
                let (value, result) = BorderParams::from_table(
                    value, &self.palette
                );
                match field {
                    BorderField::App => self.border = Some(value),
                    BorderField::Dialog => self.dialog_border = value,
                }
                result
            }
            (StyleConfigField::Text(field), Value::Table(value)) => {
                let (value, result) = TextParams::from_table(
                    value, &self.palette
                );
                match field {
                    StyleTextField::General       => self.general = value,
                    StyleTextField::Banner        => self.banner = value,
                    StyleTextField::Footer        => self.footer = value,
                    StyleTextField::DialogBody    => self.dialog_body = value,
                    StyleTextField::DialogHeading => self.dialog_prompt = value,
                    StyleTextField::Text          => self.text = value,
                    StyleTextField::Heading3      => self.heading3 = value,
                    StyleTextField::Heading2      => self.heading2 = value,
                    StyleTextField::Heading1      => self.heading1 = value,
                    StyleTextField::Preformat     => self.preformat = value,
                    StyleTextField::Link          => self.link = value,
                    StyleTextField::Error         => self.error = value,
                    StyleTextField::Quote         => self.quote = value,
                    StyleTextField::List          => self.list = value,
                    
                }
                result
            }
            (StyleConfigField::Margin(field), Value::Table(value)) => {
                let (value, result) = MarginParams::from_table(value, &());
                match field {
                    StyleMarginField::Text         => self.text_margin = value,
                    StyleMarginField::Screen       => self.screen_margin = value,
                    StyleMarginField::DialogText   => self.dialog_text_margin = value,
                    StyleMarginField::DialogScreen => self.dialog_screen_margin = value,
                }
                result
            }
            (field, value) => Err(format!("
                field {field:?} value {value:?} not valid here
            ")),
        }
    }
}


pub fn parse_color(value: &toml::Value, palette: &Map<String, Value>) 
    -> Result<crossterm::style::Color, String> 
{
    match &value {
        Value::String(string) => {
            if let Some(value) = palette.get(string)
            && let Value::String(string) = value
            && let Some('#') = string.chars().next()
            {
                color::parse_hex_color(&string[1..])
            }
            else if let Some('#') = string.chars().next() {
                color::parse_hex_color(&string[1..])
            } else {
                return Err(format!("
                Color error for value `{string}`:
                    `{string}` does not refer to a variable in the palette table, 
                    nor is it a hex value (#RRGGBB). 
                "))
            }
        }
        value => return Err(format!("
                {value:?} is of a toml type that can not be used
                in a color assignment.
        "))
    }
}

impl UserAssign<Map<String, Value>> for Style {
    type Field = StyleField;

    fn assign(
        &mut self, 
        field:   Self::Field, 
        value:   Value, 
        palette: &Map<String, Value>

    ) -> Result<(), String> 
    {
        match (field, value) {
            (StyleField::Color(field), value) => {
                let value = parse_color(&value, palette)
                    .map_err(|e| format!("{field:?}\n{e}"))?;
                match field {
                    ColorField::Fg => self.fg = Some(value),
                    ColorField::Bg => self.bg = Some(value),
                }
            }
            (StyleField::Attribute(field), Value::Boolean(value)) => {
                match field {
                    AttributeField::Bold      => self.bold = value,
                    AttributeField::Underline => self.underline = value,
                }
            }
            (field, value) => return Err(format!("
                field {field:?} value {value:?} not valid here
            ")),
        }
        Ok(())
    }
}

impl UserAssign<()> for MarginParams {
    type Field = MarginParamsField;

    fn assign(
        &mut self, field: Self::Field, value: Value, _: &()
    ) -> Result<(), String> {

        match (field, value) {
            (field, Value::Integer(value)) => {
                let value = u16::try_from(value).map_err(
                    |e| format!("{value:?} : {e}")
                )?;
                match field {
                    MarginParamsField::North => self.north = value,
                    MarginParamsField::South => self.south = value,
                    MarginParamsField::East  => self.east = value,
                    MarginParamsField::West  => self.west = value,
                }
            }
            (_, value) => return Err(format!("
                Margin must be a number, not {value:?}
            ")),
        }
        Ok(())
    }
}

impl UserAssign<Map<String, Value>> for BorderParams {
    type Field = BorderParamsField;

    fn assign(
        &mut self, field: Self::Field, value: Value, ctx: &Map<String, Value>
    ) -> Result<(), String> {

        match (field, value) {
            (BorderParamsField::Style(field), value) => {
                self.style.assign(field, value, ctx)?;
            }
            (BorderParamsField::Corner, Value::String(value)) => {
                match value.as_str() {
                    "square" => {
                        self.northwest = util::NW_SQR;
                        self.northeast = util::NE_SQR;
                        self.southwest = util::SW_SQR;
                        self.southeast = util::SE_SQR;
                    }
                    "round" => {
                        self.northwest = util::NW_RND;
                        self.northeast = util::NE_RND;
                        self.southwest = util::SW_RND;
                        self.southeast = util::SE_RND;
                    }
                    value => return Err(format!("
                        Corner field does not contain {value}
                    ")),
                }
            }
            (BorderParamsField::Bracket, Value::String(value)) => {
                match value.as_str() {
                    "space" => {
                        self.open = ' ';
                        self.close = ' ';
                    }
                    "tortoise" | "tort" | "t" => {
                        self.open = util::OPEN_TORT;
                        self.close = util::CLOSE_TORT;
                    }
                    "integral" | "int"  | "i" | "j" | "J" => {
                        self.open = util::OPEN_INT;
                        self.close = util::CLOSE_INT;
                    }
                    "square" | "sqr" => {
                        self.open = util::OPEN_SQR;
                        self.close = util::CLOSE_SQR;
                    }
                    "E" | "e" => {
                        self.open = util::OPEN_E;
                        self.close = util::CLOSE_E;
                    }
                    value => return Err(format!("
                        Bracket field does not contain {value}
                    ")),
                }
            }
            (field, value) => return Err(format!("
                field {field:?} value {value:?} not valid here
            ")),
        }
        Ok(())
    }
}

impl UserAssign<Map<String, Value>> for TextParams {
    type Field = TextStyleParamsField;

    fn assign(
        &mut self, field: Self::Field, value: Value, ctx: &Map<String, Value>
    ) -> Result<(), String> {

        match (field, value) {
            (TextStyleParamsField::Wrap, Value::Boolean(value)) => {
                self.wrap = value;
            }
            (TextStyleParamsField::Style(field), value) => {
                self.style.assign(field, value, ctx)?;
            }
            (field, value) => return Err(format!("
                field {field:?} value {value:?} not valid here
            ")),
        }
        Ok(())
    }
}
