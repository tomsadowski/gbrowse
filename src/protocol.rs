// src/network.rs

use url::{Url, ParseError};
use native_tls::TlsConnector;
use std::{
  thread,
  sync::mpsc,
  time::Duration, 
  io::{Write, Read},
  net::{TcpStream, ToSocketAddrs},
};


pub struct Request {
  pub url:    Url,
  pub rx:     mpsc::Receiver<Result<(String, String), String>>,
  pub handle: thread::JoinHandle<()>,
}
impl Request {
  pub fn new(url: &Url, timeout: u64) -> Self {
    let (tx, rx)  = mpsc::channel::<Result<(String, String), String>>();
    let url_clone = url.clone();
    let handle    = thread::spawn(
      move || {
        let result = get_data(&url_clone, timeout);
        tx.send(result).unwrap();
      });
    Self {url: url.clone(), rx, handle}
  }
}

pub fn split_whitespace_once(line: &str) -> Option<(&str, &str)> {
  line.find('\u{0009}').or(line.find(' '))
    .map(|i| (line[..i].trim(), line[i..].trim()))
}
pub fn join_if_relative(base: &Url, url_str: &str) -> Result<Url, ParseError> {
  Url::parse(url_str).or_else(|e|
    if let ParseError::RelativeUrlWithoutBase = e {
      base.join(url_str)
    } else {
      Err(e)
    }
  )
}
pub struct GemDoc {
  pub status: StatusText,
  pub doc:    Vec<GemText>,
}
impl GemDoc {
  pub fn new(url: &Url, response: String, content: String) -> Result<Self, String> {
    let status = StatusText::parse(&response);
    let doc = match status.tag {
      Status::Success => GemText::parse_doc(&content, url),
      _ => {
        let msg = format!("status: {:?}, text: {}", status.tag, status.txt);
        vec![GemText::new(GemTag::Text, &msg)]
      }
    };
    Ok(Self {status, doc})
  }
}
#[derive(Clone, PartialEq, Debug)]
pub struct GemText {
  pub tag: GemTag,
  pub txt: String,
}
impl GemText {
  pub fn new(tag: GemTag, txt: &str) -> Self {
    Self {tag, txt: String::from(txt)}
  }
  pub fn parse_doc(text_str: &str, source: &Url) -> Vec<Self> {
    let mut vec = vec![];
    let mut preformat = false;
    for line in text_str.lines() {
      if let Some(("```", _)) = line.split_at_checked(3) {
        preformat = !preformat;
      } else if preformat {
        vec.push(Self::new(GemTag::PreFormat, line.into()));
      } else {
        vec.push(Self::parse_formatted(line, source));
      }
    }
    vec
  }
  pub fn parse_formatted(line: &str, source: &Url) -> Self {
    // look for 3 character symbols
    if let Some(("###", text)) = line.split_at_checked(3) {
      let text = text.trim();
      return Self::new(GemTag::HeadingThree, text.into())
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
      let text = text.trim();
      if symbol == "=>" {
        let (url_str, link_str) = 
          split_whitespace_once(text).unwrap_or((text, text));
        match join_if_relative(source, url_str) {
          Ok(url) =>
            return Self::new(
              GemTag::Link(Scheme::from(&url), url), 
              link_str.into()),
          Err(s) => 
            return Self::new(
              GemTag::BadLink(s.to_string()), 
              link_str.into())
        }
      } else if symbol == "##" {
        return Self::new(GemTag::HeadingTwo, text.into())
      }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
      let text = text.trim();
      if symbol == ">" {
        return Self::new(GemTag::Quote, text.into())
      } else if symbol == "*" {
        return Self::new(GemTag::ListItem, &format!("- {}", text))
      } else if symbol == "#" {
        return Self::new(GemTag::HeadingOne, text.into())
      }
    }
    return Self::new(GemTag::Text, line.into())
  }
}
#[derive(Clone, PartialEq, Debug)]
pub enum GemTag {
  HeadingOne,
  HeadingTwo,
  HeadingThree,
  Text, 
  PreFormat,
  Link(Scheme, Url),
  BadLink(String),
  ListItem,
  Quote,
} 
#[derive(Debug, Clone)]
pub struct StatusText {
  pub tag: Status, 
  pub txt: String,
}
impl StatusText {
  pub fn parse(line: &str) -> Self {
    let line = line.trim();
    let (code_str, msg) = split_whitespace_once(line).unwrap_or((line, line));
    let tag = Status::from(code_str);
    Self {tag, txt: msg.into()}
  }
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
#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {
  Gemini, 
  Gopher, 
  Http, 
  Unknown
}
impl From<&Url> for Scheme {
  fn from(url: &Url) -> Scheme {
    match url.scheme() {
      "gemini" => Scheme::Gemini,
      "gopher" => Scheme::Gopher,
      "http"   => Scheme::Http,
      "https"  => Scheme::Http,
      _        => Scheme::Unknown,
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
