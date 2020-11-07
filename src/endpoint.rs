use crate::{Body, Request, Response, Service};
use futures::future::BoxFuture;

pub(crate) type EndpointService = Box<
  dyn Service<
      Request,
      Response = Response<Body>,
      Error = hyper::Error,
      Future = BoxFuture<'static, Result<Response<Body>, hyper::Error>>,
    > + Send
    + Sync,
>;

/// An HTTP routing endpoint.
pub struct Endpoint {
  /// the handler service.
  handler: EndpointService,

  /// the routing pattern used for handler nodes
  pattern: String,

  /// the parameter keys recorded on handler nodes
  param_keys: Vec<String>,
}

