// src/util.rs


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

