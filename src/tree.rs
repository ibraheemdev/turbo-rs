use crate::{Method, Endpoint};
use crate::endpoint::EndpointService;
use std::collections::HashMap;
use regex::Regex;

/// The types of nodes the tree can hold
pub enum NodeType {
  /// A Regex URL parameter
  Regex,
  /// A URL parameter, ex: `/:id`. See `Param`
  Param,
  /// A wilcard parameter, ex: `/*static`
  CatchAll,
  /// Anything else
  Static
}

pub struct Node {
  typ: NodeType,
  label: u8,
  tail: u8,
  prefix: String,
  regex: Regex,
  endpoints: HashMap<Method, Endpoint>
}

impl Node {
  fn insert_route(&self, method: Method, pattern: String, service: EndpointService) -> Self {
    let parent: Node;
    let current_node: Node;
    let search = pattern.clone();
    
    loop {
      // Handle key exhaustion
      if search.is_empty() {
        // Insert or update the node's leaf handler
        self.set_endpoint(method, service, pattern);
        return current_node;
      }
    }
  }
}