#![deny(clippy::all)]
#![forbid(unsafe_code)]

// pub mod app;
pub mod extractors;
pub mod handler;
pub mod responder;
pub mod endpoint;
pub mod tree;
pub mod service;

// re-export common components
// #[doc(hidden)]
// pub use app::App;
#[doc(hidden)]
pub use extractors::FromRequest;
#[doc(hidden)]
pub use http::{HeaderValue, Method};
#[doc(hidden)]
pub use hyper::{Body, Error, Response, StatusCode};
#[doc(hidden)]
pub use responder::ToResponse;
#[doc(hidden)]
pub use endpoint::Endpoint;
#[doc(hidden)]
pub use service::Service;

// #[macro_use]
// extern crate log;

/// `Request` is a `hyper::Request` wrapped in a reference-counting pointer.
/// The request needs to be wrapped in a pointer in order to be passed around
/// and cloned throughout the `turbofish::Service` chain
#[derive(Clone)]
pub struct Request(pub std::sync::Arc<hyper::Request<hyper::Body>>);

impl Request {
  fn new(from: hyper::Request<hyper::Body>) -> Self {
    Request(std::sync::Arc::from(from))
  }
}
