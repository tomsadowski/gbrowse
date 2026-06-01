// src/network.rs

use url::{Url, ParseError as UrlParseError};
use native_tls::TlsConnector;
use std::{
  thread,
  sync::mpsc,
  time::Duration, 
  io::{Write, Read},
  net::{TcpStream, ToSocketAddrs},
};


pub fn split_whitespace_once(line: &str) -> Option<(&str, &str)> {
  line
    .find('\u{0009}')
    .or(line.find(' '))
    .map(|i| (line[..i].trim(), line[i..].trim()))
}

pub fn join_if_relative(base: &Url, url_str: &str) 
  -> Result<Url, UrlParseError> 
{
  Url::parse(url_str).or_else(|e|
    if let UrlParseError::RelativeUrlWithoutBase = e {
      base.join(url_str)
    } else {
      Err(e)
    }
  )
}

#[derive(Debug, Clone)]
pub enum Status {
  InputExpected,
  InputExpectedSensitive,
  Success,
  RedirectTemporary,
  RedirectPermanent,
  FailTemporary,
  FailServerUnavailable,
  FailCGIError,
  FailProxyError,
  FailSlowDown,
  FailPermanent,
  FailNotFound,             
  FailGone,                 
  FailProxyRequestRefused,  
  FailBadRequest,           
  CertRequiredClient,
  CertRequiredTransient,   
  CertRequiredAuthorized,  
  CertNotAccepted,         
  FutureCertRejected,      
  ExpiredCertRejected,     
  Unknown(u8),
  Junk(String),
}
impl From<&str> for Status {
  fn from(item: &str) -> Status {
    match item.parse::<u8>().map_err(|e| e.to_string()) {
      Ok(u) => match u {
        10 | 12..=19 => Status::InputExpected,
        11 =>           Status::InputExpectedSensitive,
        20..=29 =>      Status::Success,
        30 | 32..=39 => Status::RedirectTemporary,
        31 =>           Status::RedirectPermanent,
        41 =>           Status::FailServerUnavailable,
        40 | 45..=49 => Status::FailTemporary,
        42 =>           Status::FailCGIError,
        43 =>           Status::FailProxyError,
        44 =>           Status::FailSlowDown,
        50 | 54..=58 => Status::FailPermanent,
        51 =>           Status::FailNotFound,
        52 =>           Status::FailGone,
        53 =>           Status::FailProxyRequestRefused,
        59 =>           Status::FailBadRequest,
        60 | 66..=69 => Status::CertRequiredClient,
        61 =>           Status::CertRequiredTransient,
        62 =>           Status::CertRequiredAuthorized,
        63 =>           Status::CertNotAccepted,
        64 =>           Status::FutureCertRejected,
        65 =>           Status::ExpiredCertRejected,
        u =>            Status::Unknown(u),
      } 
      Err(e) => Status::Junk(e)
    }
  }
}
#[derive(Debug, Clone)]
pub struct StatusText {
  pub tag:  Status, 
  pub text: String,
}
impl StatusText {
  pub fn parse(line: &str) -> Self {
    let line = line.trim();
    let (code_str, msg) = 
      split_whitespace_once(line).unwrap_or((line, line));
    Self {
      tag: Status::from(code_str), 
      text: msg.into()
    }
  }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GemTag {
  HeadingOne,
  HeadingTwo,
  HeadingThree,
  Text, 
  PreFormat,
  Link(String),
  ListItem,
  Quote,
} 

#[derive(Clone, PartialEq, Debug)]
pub struct GemText {
  pub tag:  GemTag,
  pub text: String,
}
impl std::fmt::Display for GemText {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) 
    -> Result<(), std::fmt::Error> 
  {
    write!(f, "{}", self.text)
  }
}
impl From<(GemTag, String)> for GemText {
  fn from(item: (GemTag, String)) -> Self {
    Self {tag: item.0, text: item.1}
  }
}
impl GemText {
  pub fn text(s: String) -> Self {
    Self {tag: GemTag::Text, text: s}
  }
  pub fn heading3(s: String) -> Self {
    Self {tag: GemTag::HeadingThree, text: s}
  }
  pub fn heading2(s: String) -> Self {
    Self {tag: GemTag::HeadingTwo, text: s}
  }
  pub fn heading1(s: String) -> Self {
    Self {tag: GemTag::HeadingOne, text: s}
  }
  pub fn preformat(s: String) -> Self {
    Self {tag: GemTag::PreFormat, text: s}
  }
  pub fn quote(s: String) -> Self {
    Self {tag: GemTag::Quote, text: s}
  }
  pub fn list_item(s: String) -> Self {
    Self {tag: GemTag::ListItem, text: s}
  }

  pub fn parse_line(line: &str) -> (GemTag, String) {
    if let Some(("```", _)) = line.split_at_checked(3) {
      (GemTag::PreFormat, "".into())
    } else if let Some(("###", t)) = line.split_at_checked(3) {
      (GemTag::HeadingThree, t.into())
    } else if let Some(("##", t)) = line.split_at_checked(2) {
      (GemTag::HeadingTwo, t.trim().into())
    } else if let Some(("#", t)) = line.split_at_checked(1) {
      (GemTag::HeadingOne, t.trim().into())
    } else if let Some((">", t)) = line.split_at_checked(1) {
      (GemTag::Quote, t.trim().into())
    } else if let Some(("*", t)) = line.split_at_checked(1) {
      (GemTag::ListItem, t.into())
    } else if let Some(("=>", t)) = line.split_at_checked(2) {
      let (u, t) = split_whitespace_once(t).unwrap_or((t, t));
      (GemTag::Link(u.into()), t.trim().into())
    } else {
      (GemTag::Text, line.trim().into())
    }
  }
}

pub struct GemDoc {
  pub status: StatusText,
  pub doc:    Vec<GemText>,
}
impl GemDoc {
  pub fn new(response: String, content: String) -> Result<Self, String> {
    let status = StatusText::parse(&response);
    let doc = match status.tag {
      Status::Success => Self::parse_doc(&content),
      _ => vec![GemText::text(format!("{:?}", status))],
    };
    Ok(Self {status, doc})
  }

  pub fn parse_doc(text_str: &str) -> Vec<GemText> {
    let mut vec       = vec![];
    let mut preformat = false;
    for line in text_str.lines() {
      match (&mut preformat, GemText::parse_line(line)) {
        (_, (GemTag::PreFormat, _)) => preformat = !preformat,
        (true,  (_, s)) => vec.push(GemText::preformat(s)),
        (false, tuple)  => vec.push(tuple.into()),
      }
    }
    vec
  }
}

pub struct Request {
  pub url:    Url,
  pub rx:     mpsc::Receiver<Result<(String, String), String>>,
  pub handle: thread::JoinHandle<()>,
}
impl Request {
  pub fn new(url: &Url, timeout: u64) -> Self {
    let (tx, rx)  = mpsc::channel::<Result<(String, String), String>>();
    let url_clone = url.clone();
    let handle = thread::spawn(
      move || {
        let result = get_data(&url_clone, timeout);
        tx.send(result).unwrap();
      });
    Self {url: url.clone(), rx, handle}
  }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {
  Gemini, 
  Gopher, 
  Http, 
  Https, 
  Unknown(String),
}
impl std::str::FromStr for Scheme {
  type Err = UrlParseError;
  fn from_str(str: &str) -> Result<Self, Self::Err> {
    match Url::parse(str)?.scheme() {
      "gemini" => Ok(Scheme::Gemini),
      "gopher" => Ok(Scheme::Gopher),
      "http"   => Ok(Scheme::Http),
      "https"  => Ok(Scheme::Https),
      s        => Ok(Scheme::Unknown(s.into())),
    }
  }
}
impl ToString for Scheme {
  fn to_string(&self) -> String {
    match self {
      Scheme::Gemini     => "gemini".into(),
      Scheme::Gopher     => "gopher".into(),
      Scheme::Http       => "http".into(),
      Scheme::Https      => "https".into(),
      Scheme::Unknown(s) => s.into(),
    }
  }
}

// returns response and content
pub fn get_data(url: &Url, timeout: u64) -> Result<(String, String), String> {
  let host = url.host_str().unwrap_or("");
  let urlf = format!("{}:1965", host);
  // get connector
  let connector = TlsConnector::builder()
    .danger_accept_invalid_hostnames(true)
    .danger_accept_invalid_certs(true)
    .build()
    .map_err(|e| e.to_string())?;
  // get socket address iterator
  let mut addrs_iter = urlf
    .to_socket_addrs()
    .map_err(|e| e.to_string())?;
  // get socket address from socket address iterator
  let socket_addr = addrs_iter
    .next()
    .ok_or(format!("socket address not found for {}", urlf))?;
  // get tcp stream from socket address
  let tcpstream = 
    TcpStream::connect_timeout(&socket_addr, Duration::new(timeout, 0))
      .map_err(|e| e.to_string())?;
  // get stream from tcp stream
  let mut stream = connector
    .connect(&host, tcpstream) 
    .map_err(|e| e.to_string())?;
  // write url to stream
  stream
    .write_all(format!("{}\r\n", url).as_bytes())
    .map_err(|e| e.to_string())?;
  // read into response vector
  let mut response = vec![];
  stream
    .read_to_end(&mut response)
    .map_err(|e| e.to_string())?;
  // separate response from content
  let clrf = b"\r\n";
  let content = response
    .windows(clrf.len())
    .position(|window| window == clrf)
    .map(|idx| response.split_off(idx + 2))
    .map(|content| String::from_utf8_lossy(&content).into())
    .unwrap_or("no content".into());
  Ok((String::from_utf8_lossy(&response).into(), content))
}
