// src/gemdoc.rs

use crate::util;


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
      util::split_whitespace_once(line).unwrap_or((line, line));
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
      let (u, t) = util::split_whitespace_once(t.trim()).unwrap_or((t, t));
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
