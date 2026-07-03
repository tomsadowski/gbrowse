// src/network.rs

use std::{sync, thread};

pub struct Request {
  pub url:    url::Url,
  pub rx:     sync::mpsc::Receiver<Result<(String, String), String>>,
  pub handle: thread::JoinHandle<()>,
}

impl Request {
  pub fn new(url: &url::Url, timeout: u64) -> Self {
    use sync::mpsc;
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

// returns response and content
pub fn get_data(url: &url::Url, timeout: u64) 
-> Result<(String, String), String> 
{
  use std::{
    time::Duration, 
    io::{Write, Read},
    net::{TcpStream, ToSocketAddrs},
  };
  let host = url.host_str().unwrap_or("");
  let urlf = format!("{}:1965", host);
  // get connector
  let connector = native_tls::TlsConnector::builder()
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
