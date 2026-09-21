
// src/userstyle.rs

use crate::{
    UserAssign, 
    UserTable,
    AssignResult,
    AssignErr,
    ValueResult,
    ValueErr,
    MarginParams,
    BorderParams,
    TextParams,
    TabText,
    GemTag,
    GemText,
    FrameParams,
    Style,
    util,
};
use toml::{Value, map::Map};
use crossterm::style::Color;



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

impl std::fmt::Display for StyleConfigField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Palette                                => write!(f, "palette"),
            Self::Border(BorderField::App)               => write!(f, "border"),
            Self::Border(BorderField::Dialog)            => write!(f, "dialog_border"),
            Self::Margin(StyleMarginField::Text)         => write!(f, "text_margin"),
            Self::Margin(StyleMarginField::Screen)       => write!(f, "screen_margin"),
            Self::Margin(StyleMarginField::DialogText)   => write!(f, "dialog_text_margin"),
            Self::Margin(StyleMarginField::DialogScreen) => write!(f, "dialog_screen_margin"),
            Self::Text(StyleTextField::General)          => write!(f, "general"),
            Self::Text(StyleTextField::Banner)           => write!(f, "banner"),
            Self::Text(StyleTextField::Footer)           => write!(f, "footer"),
            Self::Text(StyleTextField::DialogBody)       => write!(f, "dialog_body"),
            Self::Text(StyleTextField::DialogHeading)    => write!(f, "dialog_heading"),
            Self::Text(StyleTextField::Text)             => write!(f, "text"),
            Self::Text(StyleTextField::Heading3)         => write!(f, "heading3"),
            Self::Text(StyleTextField::Heading2)         => write!(f, "heading2"),
            Self::Text(StyleTextField::Heading1)         => write!(f, "heading1"),
            Self::Text(StyleTextField::Preformat)        => write!(f, "preformat"),
            Self::Text(StyleTextField::Link)             => write!(f, "link"),
            Self::Text(StyleTextField::Error)            => write!(f, "error"),
            Self::Text(StyleTextField::Quote)            => write!(f, "quote"),
            Self::Text(StyleTextField::List)             => write!(f, "list"),
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

impl std::fmt::Display for MarginParamsField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::North  => write!(f, "north"),
            Self::South  => write!(f, "south"),
            Self::East   => write!(f, "east"),
            Self::West   => write!(f, "west"),
        }
    }
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

impl std::fmt::Display for TextStyleParamsField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Wrap          => write!(f, "wrap"),
            Self::Style(style)  => style.fmt(f),
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
            string      => StyleField::from_str(string).map(|s| Self::Style(s))
        }
    }
}

impl std::fmt::Display for BorderParamsField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Corner        => write!(f, "corner"),
            Self::Bracket       => write!(f, "bracket"),
            Self::Style(style)  => style.fmt(f),
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

impl std::fmt::Display for StyleField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Color(ColorField::Fg)                => write!(f, "fg"),
            Self::Color(ColorField::Bg)                => write!(f, "bg"),
            Self::Attribute(AttributeField::Bold)      => write!(f, "bold"),
            Self::Attribute(AttributeField::Underline) => write!(f, "underline"),
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
        &mut self, field: &Self::Field, value: Value, _: &()
    ) -> AssignResult {

        match (field, value) {
            (StyleConfigField::Border(border), Value::Table(value)) => {
                let (string, result) = BorderParams::from_table(
                    value, &self.palette
                );
                match border {
                    BorderField::App => self.border = Some(string),
                    BorderField::Dialog => self.dialog_border = string,
                }
                result.map_err(|e| AssignErr::new(
                    field, ValueErr::Message(e)
                ))
            }
            (StyleConfigField::Text(text), Value::Table(value)) => {
                let (text_params, result) = TextParams::from_table(
                    value, &self.palette
                );
                match text {
                    StyleTextField::General       => self.general = text_params,
                    StyleTextField::Banner        => self.banner = text_params,
                    StyleTextField::Footer        => self.footer = text_params,
                    StyleTextField::DialogBody    => self.dialog_body = text_params,
                    StyleTextField::DialogHeading => self.dialog_prompt = text_params,
                    StyleTextField::Text          => self.text = text_params,
                    StyleTextField::Heading3      => self.heading3 = text_params,
                    StyleTextField::Heading2      => self.heading2 = text_params,
                    StyleTextField::Heading1      => self.heading1 = text_params,
                    StyleTextField::Preformat     => self.preformat = text_params,
                    StyleTextField::Link          => self.link = text_params,
                    StyleTextField::Error         => self.error = text_params,
                    StyleTextField::Quote         => self.quote = text_params,
                    StyleTextField::List          => self.list = text_params,
                }
                result.map_err(|e| AssignErr::new(
                    field, ValueErr::Message(e)
                ))
            }
            (StyleConfigField::Margin(margin), Value::Table(value)) => {
                let (value, result) = MarginParams::from_table(value, &());
                match margin {
                    StyleMarginField::Text         => self.text_margin = value,
                    StyleMarginField::Screen       => self.screen_margin = value,
                    StyleMarginField::DialogText   => self.dialog_text_margin = value,
                    StyleMarginField::DialogScreen => self.dialog_screen_margin = value,
                }
                result.map_err(|e| AssignErr::new(
                    field, ValueErr::Message(e)
                ))
            }
            (field, value) => Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            )),
        }
    }
}


pub fn parse_hex_color(s: &str) -> Result<Color, String> {
    fn try_hex(c: char) -> Result<u8, String> {
        match c {
            '0' => Ok(0),  '1' => Ok(1),  '2' => Ok(2),  '3' => Ok(3),
            '4' => Ok(4),  '5' => Ok(5),  '6' => Ok(6),  '7' => Ok(7),
            '8' => Ok(8),  '9' => Ok(9),  'a' => Ok(10), 'b' => Ok(11),
            'c' => Ok(12), 'd' => Ok(13), 'e' => Ok(14), 'f' => Ok(15),
            _   => Err(format!("{c} is not a hex character")),
        }
    }
    let mut c = s.chars();
    let r1 = c
        .next()
        .ok_or("Missing 6 hex characters.".into())
        .and_then(|c| try_hex(c))?;
    let r2 = c
        .next()
        .ok_or("Missing 5 hex characters.".into())
        .and_then(|c| try_hex(c))?;
    let g1 = c
        .next()
        .ok_or("Missing 4 hex characters.".into())
        .and_then(|c| try_hex(c))?;
    let g2 = c
        .next()
        .ok_or("Missing 3 hex characters.".into())
        .and_then(|c| try_hex(c))?;
    let b1 = c
        .next()
        .ok_or("Missing 2 hex characters.".into())
        .and_then(|c| try_hex(c))?;
    let b2 = c
        .next()
        .ok_or("Missing 1 hex character.".into())
        .and_then(|c| try_hex(c))?;

    let r = 16 * r1 + r2;
    let g = 16 * g1 + g2;
    let b = 16 * b1 + b2;

    Ok(Color::Rgb {r, g, b})
}

pub fn parse_color(value: &toml::Value, palette: &Map<String, Value>) 
    -> ValueResult<Color> 
{
    match &value {
        Value::String(string) => {
            if let Some(value) = palette.get(string)
            && let Value::String(string) = value
            && let Some('#') = string.chars().next()
            {
                parse_hex_color(&string[1..])
                    .map_err(|e| ValueErr::InvalidParse(e))
            }
            else if let Some('#') = string.chars().next() {
                parse_hex_color(&string[1..])
                    .map_err(|e| ValueErr::InvalidParse(e))
            } else {
                return Err(ValueErr::InvalidParse(format!("
                    `{string}` does not refer to a variable in the palette table, nor is it a hex value.
                ")))
            }
        }
        value => return Err(ValueErr::InvalidTomlType((*value).clone()))
    }
}

impl UserAssign<Map<String, Value>> for Style {
    type Field = StyleField;

    fn assign(
        &mut self, 
        field:   &Self::Field, 
        value:   Value, 
        palette: &Map<String, Value>

    ) -> AssignResult {

        match (field, value) {
            (StyleField::Color(color), value) => {
                let value = parse_color(&value, palette)
                    .map_err(|e| AssignErr::new(field, e))?;
                match color {
                    ColorField::Fg => self.fg = Some(value),
                    ColorField::Bg => self.bg = Some(value),
                }
            }
            (StyleField::Attribute(attr), Value::Boolean(value)) => {
                match attr {
                    AttributeField::Bold      => self.bold = value,
                    AttributeField::Underline => self.underline = value,
                }
            }
            (field, value) => return Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            )),
        }
        Ok(())
    }
}

impl UserAssign<()> for MarginParams {
    type Field = MarginParamsField;

    fn assign(
        &mut self, field: &Self::Field, value: Value, _: &()
    ) -> AssignResult {

        match (field, value) {
            (field, Value::Integer(value)) => {
                let value = u16::try_from(value).map_err(
                    |e| AssignErr::new(
                        field, ValueErr::InvalidParse(e.to_string())
                    )
                )?;
                match field {
                    MarginParamsField::North => self.north = value,
                    MarginParamsField::South => self.south = value,
                    MarginParamsField::East  => self.east = value,
                    MarginParamsField::West  => self.west = value,
                }
            }
            (field, value) => return Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            )),
        }
        Ok(())
    }
}

impl UserAssign<Map<String, Value>> for BorderParams {
    type Field = BorderParamsField;

    fn assign(
        &mut self, field: &Self::Field, value: Value, ctx: &Map<String, Value>
    ) -> AssignResult {

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
                    value => return Err(AssignErr::new(
                        field, ValueErr::InvalidParse(value.into())
                    )),
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
                    value => return Err(AssignErr::new(
                        field, ValueErr::InvalidParse(value.into())
                    )),
                }
            }
            (field, value) => return Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            )),
        }
        Ok(())
    }
}

impl UserAssign<Map<String, Value>> for TextParams {
    type Field = TextStyleParamsField;

    fn assign(
        &mut self, field: &Self::Field, value: Value, ctx: &Map<String, Value>
    ) -> AssignResult {

        match (field, value) {
            (TextStyleParamsField::Wrap, Value::Boolean(value)) => {
                self.wrap = value;
            }
            (TextStyleParamsField::Style(field), value) => {
                self.style.assign(field, value, ctx)?;
            }
            (field, value) => return Err(AssignErr::new(
                field, ValueErr::InvalidTomlType(value)
            )),
        }
        Ok(())
    }
}
