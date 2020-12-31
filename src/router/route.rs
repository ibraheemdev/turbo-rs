use crate::router::{Config, Match};
use crate::{Action, Request};
use std::collections::HashMap;

pub trait Matcher {
  fn match_req(&self, req: Request, route_match: Match) -> bool;
}

pub type BuildVarsFunc = Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>;

// Route stores information to match a request and build URLs.
pub struct Route<'a> {
  // Request handler for the route.
  handler: Box<dyn Action>,

  // If true, this route never matches: it is only used to build URLs.
  build_only: bool,

  // The name used to build URLs.
  name: String,

  // "global" reference to all named routes
  named_routes: HashMap<String, &'a Route<'a>>,

  // config possibly passed in from `Router`
  config: Config,
}
