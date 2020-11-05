use crate::{Request, Route};
use http::Response;
use std::collections::HashMap;

/// Registers routes to be matched and dispatches a handler.
///
/// It implements the http.Handler interface, so it can be registered to serve
/// requests:
/// ```
/// let router = Router::Default();
/// router.serve().await;
/// ```
/// This will send all incoming requests to the router.
pub struct Router<'a> {
  /// Configurable Handler to be used when no route matches.
  not_found_handler: Option<Route<'a>>,

  /// Configurable Handler to be used when the request method does not match the route.
  method_not_found_handler: Option<Route<'a>>,

  /// Routes to be matched, in order.
  routes: Vec<Route<'a>>,

  /// Routes by name for URL building.
  named_routes: HashMap<String, Route<'a>>,

  /// Middlewares to be called after a match is found
  middlewares: Vec<Route<'a>>,

  /// Configuration shared with `Route`
  config: RouteConfig,
}

impl<'a> Default for Router<'a> {
  /// Returns the default router instance.
  fn default() -> Self {
    Router {
      not_found_handler: None,
      method_not_found_handler: None,
      routes: Vec::new(),
      named_routes: HashMap::new(),
      middlewares: Vec::new(),
      config: RouteConfig::default(),
    }
  }
}

#[derive(Default, Clone)]
/// Common route configuration shared between `Router` and `Route`
pub struct RouteConfig {
  /// If true, "/path/foo%2Fbar/to" will match the path "/path/{var}/to"
  use_encoded_path: bool,

  /// If true, when the path pattern is "/path/", accessing "/path" will
  /// redirect to the former and vice versa.
  strict_slash: bool,

  /// If true, when the path pattern is "/path//to", accessing "/path//to"
  /// will not redirect
  skip_clean: bool,

  /// Manager for the variables from host and path.
  regexp: RouteRegexpGroup,

  /// List of matchers.
  matchers: Vec<Matcher>,

  /// The scheme used when building URLs.
  build_scheme: String,

  build_vars_func: BuildVarsFunc,
}

#[derive(Clone)]
pub struct Matcher {}

#[derive(Default, Clone)]
pub struct RouteRegexpGroup {}

#[derive(Default, Clone)]
pub struct BuildVarsFunc {}

pub enum MatchError {
  /// Returned when the method in the request does not match
  /// the method defined against the route.
  MethodMismatch,
  /// Returned when no route match is found.
  NotFound,
}

impl<'a> Router<'a> {
  /// Attempts to match the given request against the router's registered routes.
  ///
  /// If the request matches a route of this router or one of its subrouters the Route,
  /// Handler, and Vars fields of the the match argument are filled and this function
  /// returns true.
  ///
  /// If the request does not match any of this router's or its subrouters' routes
  /// then this function returns false. If available, a reason for the match failure
  /// will be filled in the match argument's MatchErr field. If the match failure type
  /// (eg: not found) has a registered handler, the handler is assigned to the Handler
  /// field of the match argument.
  pub fn match_request(&self, req: Request) -> Result<RouteMatch, MatchError> {
    for route in &self.routes {}
    Err(MatchError::NotFound)
  }

  /// Dispatches the handler registered in the matched route.
  ///
  /// When there is a match, the route variables can be retrieved calling
  /// `router.vars(request)`.
  pub async fn serve(&self, req: Request) -> Result<Response<hyper::Body>, ()> {
    Err(())
  }

  /// Returns a route registered with the given name.
  pub fn get(&self, name: &str) -> Option<&Route> {
    self.named_routes.get(name)
  }

  /// Defines the trailing slash behavior for new routes. The initial
  /// value is false.
  ///
  /// When true, if the route path is "/path/", accessing "/path" will perform a redirect
  /// to the former and vice versa. In other words, your application will always
  /// see the path as specified in the route.
  ///
  /// When false, if the route path is "/path", accessing "/path/" will not match
  /// this route and vice versa.
  ///
  /// The re-direct is a HTTP 301 (Moved Permanently). Note that when this is set for
  /// routes with a non-idempotent method (e.g. POST, PUT), the subsequent re-directed
  /// request will be made as a GET by most clients. Use middleware or client settings
  /// to modify this behaviour as needed.
  ///
  /// Special case: when a route sets a path prefix using the `path_prefix` method,
  /// strict slash is ignored for that route because the redirect behavior can't
  /// be determined from a prefix alone. However, any subrouters created from that
  /// route inherit the original strict_slash setting.
  pub fn strict_slash(&mut self, val: bool) -> &Self {
    self.config.strict_slash = val;
    self
  }

  /// Defines the path cleaning behaviour for new routes. The initial
  /// value is false. Users should be careful about which routes are not cleaned
  ///
  /// When true, if the route path is "/path//to", it will remain with the double
  /// slash. This is helpful if you have a route like: /fetch/http://xkcd.com/534/
  ///
  /// When false, the path will be cleaned, so /fetch/http://xkcd.com/534/ will
  /// become /fetch/http/xkcd.com/534
  pub fn skip_clean(&mut self, val: bool) -> &Self {
    self.config.skip_clean = val;
    self
  }

  // Tells the router to match the encoded original path
  // to the routes.
  // For eg. "/path/foo%2Fbar/to" will match the path "/path/{var}/to".
  //
  // If not called, the router will match the unencoded path to the routes.
  // For eg. "/path/foo%2Fbar/to" will match the path "/path/foo/bar/to"
  pub fn use_encoded_path(&mut self) -> &Self {
    self.config.use_encoded_path = true;
    self
  }

  // Registers an empty route.
  pub fn new_route(&'a mut self) -> &Route {
    let route = Route::default().config(self.config.clone());
    self.routes.push(route);
    &self.routes.last().unwrap()
  }
}

/// Stores information about a matched route.
pub struct RouteMatch<'a> {
  route: &'a Route<'a>,
  vars: HashMap<String, String>,
}
