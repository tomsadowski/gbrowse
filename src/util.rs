// src/util.rs


// square
pub const NW_SQR: char = '\u{250C}';
pub const NE_SQR: char = '\u{2510}';
pub const SW_SQR: char = '\u{2514}';
pub const SE_SQR: char = '\u{2518}';

// round
pub const NW_RND: char = '\u{256D}';
pub const NE_RND: char = '\u{256E}';
pub const SW_RND: char = '\u{2570}';
pub const SE_RND: char = '\u{256F}';

// lines
pub const X_LINE: char = '\u{2500}';
pub const Y_LINE: char = '\u{2502}';

// tortoise shell square bracket (hot)
pub const OPEN_TORT: char = '\u{2997}';
pub const CLOSE_TORT: char = '\u{2998}';

// super square bracket (hot)
pub const OPEN_SQR: char = '\u{27E6}';
pub const CLOSE_SQR: char = '\u{27E7}';

// brack with quill (pretty good)
pub const OPEN_E: char = '\u{2045}';
pub const CLOSE_E: char = '\u{2046}';

// integrals (not bad)
pub const OPEN_INT: char = '\u{2320}';
pub const CLOSE_INT: char = '\u{2321}';

// ceiling / floor (not bad)
pub const OPEN_L: char = '\u{2308}';
pub const CLOSE_L: char = '\u{230B}';

pub const MANUAL: &str = "User manual";
pub const CHANGE_KEYS: &str = "Change keys";
pub const CHANGE_STYLE: &str = "Change style";
pub const VIEW_SETTINGS: &str = "View settings";
pub const MENU: [&str; 4] = [
    MANUAL, 
    CHANGE_KEYS, 
    CHANGE_STYLE,
    VIEW_SETTINGS, 
];

pub const DATA_PATH: &str = "gdata";
pub const SAVE_FILE: &str = "gdata/urls";
pub const INIT_FILE: &str = "gdata/init";
pub const STYLES_PATH: &str = "gdata/styles";
pub const KEYS_PATH: &str = "gdata/keys";


pub fn get_init_file(f: &str) -> String {
    format!("{DATA_PATH}/{f}")
}

pub fn get_keys_file(f: &str) -> String {
    format!("{KEYS_PATH}/{f}")
}

pub fn get_styles_file(f: &str) -> String {
    format!("{STYLES_PATH}/{f}")
}

pub fn split_whitespace_once(line: &str) -> Option<(&str, &str)> {
    line
        .find('\u{0009}')
        .or(line.find(' '))
        .map(|i| (line[..i].trim(), line[i..].trim()))
}

pub fn join_if_relative(base: &url::Url, url_str: &str) 
    -> Result<url::Url, url::ParseError> 
{
    url::Url::parse(url_str).or_else(|e|
        if let url::ParseError::RelativeUrlWithoutBase = e {
            base.join(url_str)
        } else {Err(e)}
    )
}

pub fn get_entries(path: &str) -> Result<Vec<String>, String> {
    let mut vec = vec![];
    for result in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        vec.push(result
            .map_err(|e| e.to_string())?
            .file_name()
            .into_string()
            .map_err(|_| "Could not convert OsString to String".to_string())?
        );
    }
    Ok(vec)
}

pub fn get_wrapped_text(input: &str, width: usize) -> Vec<Vec<char>> {
    use unicode_width::UnicodeWidthChar;
    let input: Vec<_> = input.chars().collect();
    let mut output: Vec<_> = vec![];
    let mut start = 0;
    while start < input.len() {
        let mut accum_width  = 0;
        let mut text: Vec<_> = vec![];
        let mut chars = input[start..].iter();
        while let Some(c) = chars.next() && accum_width < width {
            accum_width += &c.width().unwrap_or(0);
            text.push(c.clone());
        }
        let line: Vec<_> = {
            let s: Vec<_> = text
                .iter()
                .rev()
                .skip_while(|c| !c.is_whitespace())
                .collect();
            if text.len() < width || s.len() == 0 {
                text
            } else {
                s.into_iter().rev().copied().collect()
            }
        };
        start += line.len();
        output.push(line);
    }
    output
}
