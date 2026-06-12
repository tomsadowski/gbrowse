// src/util.rs

use crate::{
  UnitCursor,
  LineCursor,
};


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

pub trait GetWeight {
  fn get_weight(&self) -> usize;
}

impl GetWeight for char {
  fn get_weight(&self) -> usize {
    use unicode_width::UnicodeWidthChar;
    self.width().unwrap_or(0)
  }
}

pub fn get_weighted_head<U, T>(cursor: &U) -> usize 
where U: UnitCursor<Unit = T>,
      T: GetWeight,
{
  cursor
    .get_units()
    .iter()
    .take(cursor.get_head())
    .map(|u| u.get_weight())
    .sum()
}

pub fn get_weighted_length<T>(vec: &Vec<T>) -> usize 
where T: GetWeight
{
  vec
    .iter()
    .map(|u| u.get_weight())
    .sum()
}

pub fn get_weighted_view<T>(vec: &Vec<T>, axis: LineCursor) -> Vec<&T> 
where T: GetWeight
{
  let size         = usize::from(axis.get_size());  
  let mut text     = vec.iter().skip(axis.get_scroll());
  let mut acc_size = 0;
  let mut result   = vec![];
  while let Some(c) = text.next() && acc_size < size {
    acc_size += &c.get_weight();
    result.push(c);
  }
  result
}

pub fn get_wrapped_text(input: &str, width: usize) -> Vec<Vec<char>> {
  use unicode_width::UnicodeWidthChar;
  let     input:  Vec<_> = input.chars().collect();
  let mut output: Vec<_> = vec![];
  let mut start          = 0;
  while start < input.len() {
    let mut accum_width  = 0;
    let mut text: Vec<_> = vec![];
    let mut chars        = input[start..].iter();
    while let 
      Some(c) = chars.next() && 
      accum_width < width 
    {
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

#[cfg(test)]
mod util_test {
  use super::*;
  #[test]
  fn wrap_text() {
    let input = 
      "The bicycle feels good to ride so far, but I'll have to practice.";
    let output = get_wrapped_text(&input, 5);
    println!("{:?}", output);
  }
}
