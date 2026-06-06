// src/util.rs
use unicode_width::UnicodeWidthChar;


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
    } else {
      Err(e)
    }
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

pub fn wrap_text(text: Vec<char>, width: usize) -> Vec<Vec<char>> {
  let mut vec: Vec<Vec<char>> = vec![];
  let mut start = usize::MIN;
  while start < text.len() {
    let text          = &text[start..];
    let mut w         = 0;
    let mut max_width = 0;
    while w < width && max_width < text.len() {
      w         += &text[max_width].width().unwrap_or(0);
      max_width += 1;
    }
    let line: Vec<char> = {
      if text.len() <= max_width {
        text.to_vec()
      } else {
        // search for first whitespace from right
        let s: Vec<&char> = text[..max_width]
          .iter().rev().skip_while(|c| !c.is_whitespace()).collect();
        // no space found, return whole slice
        if s.len() == 0 {
          text[..max_width].iter().copied().collect()
        // space found, return up to that space
        } else {
          s.into_iter().rev().copied().collect()
        }
      }
    };
    start += line.len();
    vec.push(line);
  }
  vec
}
