use crate::config::Config;
use crate::router::Router;
use crate::server::MakeTurbofishService;
use std::net::ToSocketAddrs;
use std::time::Duration;

pub struct Turbofish {
  config: Config,
  pub(crate) router: Router
}

impl Turbofish {
  pub fn new() -> Self {
    Self {
      config: Config::default(),
      router: Router::default(),
    }
  }

  pub fn config(mut self, config: Config) -> Self {
    self.config = config;
    self
  }

  pub async fn swim(self) -> Result<(), hyper::Error> {
    let addr = format!("{}:{}", self.config.address, self.config.port)
      .to_socket_addrs()
      .map(|mut addrs| addrs.next().expect("invalid socket address"));

    hyper::server::Server::bind(&addr.expect("invalid socket address"))
      .http1_keepalive(self.config.keep_alive.is_some())
      .http2_keep_alive_interval(self.config.keep_alive.map(Duration::from_secs))
      .serve(MakeTurbofishService::new(self))
      .await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn api() {
    Turbofish::new()
      .config(Config::builder().keep_alive(100).port(3000))
      .swim();
  }
}
