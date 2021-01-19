#[doc(inline)]
pub use hyper::{Method};

/// The type of an incoming web request.
pub type Request = hyper::Request<hyper::Body>;

/// An HTTP Response.
pub type Response = hyper::Response<hyper::Body>;
