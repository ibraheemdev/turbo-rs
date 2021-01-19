pub mod tree;

use crate::http::Method;
use crate::action::BoxedAction;
use std::collections::HashMap;

pub struct Router {
  routes: HashMap<Method, BoxedAction>,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            routes: HashMap::with_capacity(5)
        }
    }
}
