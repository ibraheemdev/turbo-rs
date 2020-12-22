use std::net::{IpAddr, Ipv4Addr};

#[derive(Clone)]
pub struct Config {
  pub(crate) address: IpAddr,
  pub(crate) port: u16,
  pub(crate) keep_alive: Option<u64>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      address: Ipv4Addr::new(127, 0, 0, 1).into(),
      port: 8000,
      keep_alive: Some(5),
    }
  }
}

impl Config {
  pub fn builder() -> Self {
    Self::default()
  }

  /// Sets the keepalive timeout (default is 5)
  pub fn keep_alive(mut self, seconds: impl Into<Option<u64>>) -> Self {
    self.keep_alive = seconds.into();
    self
  }

  /// Sets the port to serve on
  pub fn port(mut self, port: u16) -> Self {
    self.port = port;
    self
  }
  /// Sets the IP address to serve on
  pub fn address(mut self, addr: impl Into<IpAddr>) -> Self {
    self.address = addr.into();
    self
  }
}
