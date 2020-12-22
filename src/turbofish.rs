use crate::config::Config;
use crate::service::MakeTurbofishService;
use std::net::SocketAddr;
use std::time::Duration;

pub struct Turbofish {
  config: Config,
}

impl Turbofish {
  pub fn new() -> Self {
    Self {
      config: Config::default(),
    }
  }

  pub fn custom(config: Config) -> Self {
    Self { config }
  }

  pub async fn swim(self, addr: &SocketAddr) -> Result<(), hyper::Error> {
    hyper::server::Server::bind(addr)
      .http1_keepalive(self.config.keep_alive.is_some())
      .http2_keep_alive_interval(self.config.keep_alive.map(Duration::from_secs))
      .serve(MakeTurbofishService::new(self))
      .await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  async fn api() -> Result<(), hyper::Error> {
    Turbofish::custom(Config::builder().keep_alive(100).port(3000))
      .swim(&([127, 0, 0, 1], 3000).into())
      .await
  }
}
