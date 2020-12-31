// tmp
#![allow(dead_code)]

pub mod action;
pub mod config;
pub mod middleware;
pub mod router;
pub mod service;
pub mod turbofish;

pub use action::Action;
pub use middleware::Middleware;

pub type Request = hyper::Request<hyper::Body>;
