pub mod regex;
pub mod route;

use self::regex::RegexGroup;
use crate::{Action, Middleware};
use route::{BuildVarsFunc, Matcher, Route};
use std::collections::HashMap;

pub struct Router<'a> {
  routes: Vec<Route<'a>>,
  named_routes: HashMap<String, String>,
  middleware: Vec<Box<dyn Middleware>>,
  config: Config,
}

pub struct Config {
  // If true, "/path/foo%2Fbar/to" will match the path "/path/{var}/to"
  use_encoded_path: bool,

  // If true, when the path pattern is "/path/", accessing "/path" will
  // redirect to the former and vice versa.
  strict_slash: bool,

  // If true, when the path pattern is "/path//to", accessing "/path//to"
  // will not redirect
  skip_clean: bool,

  // Manager for the variables from host and path.
  regex: RegexGroup,

  // List of matchers.
  matchers: Vec<Box<dyn Matcher>>,

  // The scheme used when building URLs.
  build_scheme: String,

  build_vars_func: BuildVarsFunc,
}

// RouteMatch stores information about a matched route.
pub struct Match<'a> {
  route: &'a Route<'a>,
  action: Box<dyn Action>,
  vars: HashMap<String, String>,
}
