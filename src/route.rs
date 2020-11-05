use crate::handler::{Extractor, Handler, MakeService};
use crate::router::RouteConfig;
use crate::{Body, Request, Response, Service, StatusCode};
use futures::future::{ready, BoxFuture};
use std::collections::HashMap;

type BoxedService<Req, Res> = Box<
  dyn Service<
      Req,
      Response = Res,
      Error = hyper::Error,
      Future = BoxFuture<'static, Result<Res, hyper::Error>>,
    > + Send
    + Sync,
>;

/// Stores information to match a request and build URLs.
pub struct Route<'a> {
  /// config possibly passed in from `Router`
  config: RouteConfig,

  /// "global" reference to all named routes
  named_routes: HashMap<String, &'a Route<'a>>,

  /// Request handler for the route.
  handler: BoxedService<Request, Response<hyper::Body>>,

  /// If true, this route never matches: it is only used to build URLs.
  build_only: bool,

  /// The name used to build URLs.
  name: Option<String>,
}

impl<'a> Default for Route<'a> {
  /// Returns an empty route with a not-found handler
  fn default() -> Self {
    Route {
      handler: Box::new(MakeService::new(Extractor::new(Handler::new(|| {
        ready(
          Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::default())
            .unwrap(),
        )
      })))),
      config: RouteConfig::default(),
      named_routes: HashMap::new(),
      build_only: false,
      name: None,
    }
  }
}

impl<'a> Route<'a> {
  pub fn config(mut self, config: RouteConfig) -> Self {
    self.config = config;
    self
  }
}
//   /// Create *Route* for http `GET` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::get().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn method(method: Method) -> Route {
//     Route::new().set_method(method)
//   }

//   /// Create *Route* for http `GET` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::patch().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn get() -> Route {
//     Route::new().set_method(Method::GET)
//   }

//   /// Create *Route* for http `POST` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::post().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn post() -> Route {
//     Route::new().set_method(Method::POST)
//   }

//   /// Create *Route* for http `PUT` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::put().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn put() -> Route {
//     Route::new().set_method(Method::PUT)
//   }

//   /// Create *Route* for http `PATCH` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::patch().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn patch() -> Route {
//     Route::new().set_method(Method::PATCH)
//   }

//   /// Create *Route* for http `DELETE` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::delete().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn delete() -> Route {
//     Route::new().set_method(Method::DELETE)
//   }

//   /// Create *Route* for http `HEAD` requests.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::head().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn head() -> Route {
//     Route::new().set_method(Method::HEAD)
//   }

//   /// Set handler function, use request extractors for parameters.
//   /// ```rust
//   /// use turbofish::{Route, Response, Body};
//   ///
//   /// Route::new().to(|| async {
//   ///   Response::new(Body::default())
//   /// });
//   /// ```
//   pub fn to<F, T, R, U>(mut self, handler: F) -> Self
//   where
//     F: Factory<T, R, U> + Send + Sync,
//     T: FromRequest + 'static,
//     R: Future<Output = U> + Send + Sync + 'static,
//     U: ToResponse + 'static,
//   {
//     self.handler = Box::new(MakeService::new(Extractor::new(Handler::new(handler))));
//     self
//   }

//   /// Assign the Route to an HTTP Method.
//   pub fn set_method(mut self, method: Method) -> Self {
//     self.method = Some(method);
//     self
//   }
// }
