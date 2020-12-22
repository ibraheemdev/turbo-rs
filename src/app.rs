use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::config::Config;

pub struct App {
  config: Config,
}

impl App {
  pub fn new() -> Self {
    Self {
      config: Config::default(),
    }
  }

  pub fn config(mut self, config: Config) -> Self {
    self.config = config;
    self
  }

  pub async fn run(self, addr: &SocketAddr) -> Result<(), hyper::Error> {
    let mut server = hyper::server::Server::bind(addr)
      .http1_keepalive(self.config.http1_keepalive)
      .http1_half_close(self.config.http1_half_close)
      .http1_only(self.config.http1_only)
      .http2_only(self.config.http2_only)
      .http2_initial_stream_window_size(self.config.http2_initial_stream_window_size)
      .http2_initial_connection_window_size(self.config.http2_initial_connection_window_size)
      .http2_adaptive_window(self.config.http2_adaptive_window)
      .http2_max_frame_size(self.config.http2_max_frame_size)
      .http2_max_concurrent_streams(self.config.http2_max_concurrent_streams)
      .http2_keep_alive_interval(self.config.http2_keep_alive_interval)
      .http2_keep_alive_timeout(self.config.http2_keep_alive_timeout)
      .tcp_nodelay(self.config.tcp_nodelay)
      .tcp_keepalive(self.config.tcp_keepalive)
      .tcp_sleep_on_accept_errors(self.config.tcp_sleep_on_accept_errors);

    if let Some(max) = self.config.http1_max_buf_size {
      server = server.http1_max_buf_size(max);
    }

    if let Some(writev) = self.config.http1_writev {
      server = server.http1_writev(writev);
    }

    let make_svc = MakeAppService(AppService(Arc::new(self)));
    server.serve(make_svc).await
  }
}

struct MakeAppService(AppService);

impl<T> Service<T> for MakeAppService {
  type Response = AppService;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: T) -> Self::Future {
    let service = self.0.clone();
    let fut = async move { Ok(service) };
    Box::pin(fut)
  }
}

#[derive(Clone)]
struct AppService(Arc<App>);

impl Service<Request<Body>> for AppService {
  type Response = Response<Body>;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = hyper::Result<Response<Body>>> + Send + Sync>>;

  fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: Request<Body>) -> Self::Future {
    Box::pin(async { Ok(Response::new(Body::empty())) })
  }
}


