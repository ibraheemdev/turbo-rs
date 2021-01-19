use std::cmp::min;
use std::mem;
use std::ops::Index;
use std::str;

struct Match<'a, V> {
  pub value: &'a V,
  pub params: Params,
}

#[derive(Debug, Clone, PartialEq)]
struct Param {
  pub key: String,
  pub value: String,
}

impl Param {
  fn new(key: &str, value: &str) -> Self {
    Self {
      key: key.to_string(),
      value: value.to_string(),
    }
  }
}

#[derive(Debug, PartialEq)]
struct Params(pub Vec<Param>);

impl Default for Params {
  fn default() -> Self {
    Self(Vec::new())
  }
}

impl Index<usize> for Params {
  type Output = Param;

  #[inline]
  fn index(&self, i: usize) -> &Param {
    &self.0[i]
  }
}

impl std::ops::IndexMut<usize> for Params {
  fn index_mut(&mut self, i: usize) -> &mut Param {
    &mut self.0[i]
  }
}

impl Params {
  fn by_name(&self, name: &str) -> Option<&str> {
    match self.0.iter().find(|param| param.key == name) {
      Some(param) => Some(&param.value),
      None => None,
    }
  }

  fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  fn push(&mut self, p: Param) {
    self.0.push(p);
  }
}

/// The types of nodes the tree can hold
#[derive(PartialEq, PartialOrd, Debug)]
enum NodeType {
  /// The root path
  Root,
  /// A URL parameter, ex: `/:id`. See `Param`
  Param,
  /// A wilcard parameter, ex: `/*static`
  CatchAll,
  /// Anything else
  Static,
}

struct Node<V> {
  path: Vec<u8>,
  wild_child: bool,
  node_type: NodeType,
  indices: Vec<u8>,
  children: Vec<Box<Node<V>>>,
  value: Option<V>,
  priority: u32,
}

impl<V> Default for Node<V> {
  fn default() -> Self {
    Self {
      path: Vec::new(),
      wild_child: false,
      node_type: NodeType::Static,
      indices: Vec::new(),
      children: Vec::new(),
      value: None,
      priority: 0,
    }
  }
}

impl<V> Node<V> {
  // Increments priority of the given child and reorders if necessary
  // returns the new position (index) of the child
  fn increment_child_prio(&mut self, pos: usize) -> usize {
    self.children[pos].priority += 1;
    let prio = self.children[pos].priority;
    // adjust position (move to front)
    let mut new_pos = pos;

    while new_pos > 0 && self.children[new_pos - 1].priority < prio {
      // swap node positions
      self.children.swap(new_pos - 1, new_pos);
      new_pos -= 1;
    }

    // build new index char string
    if new_pos != pos {
      self.indices = [
        &self.indices[..new_pos],    // unchanged prefix, might be empty
        &self.indices[pos..=pos],    // the index char we move
        &self.indices[new_pos..pos], // rest without char at 'pos'
        &self.indices[pos + 1..],
      ]
      .concat();
    }

    new_pos
  }

  /// Insert a `Node` with the given value to the path.
  fn insert(&mut self, path: &str, value: V) {
    let full_path = <&str>::clone(&path);
    self.priority += 1;

    // Empty tree
    if self.path.is_empty() && self.children.is_empty() {
      self.insert_child(path.as_ref(), full_path, value);
      self.node_type = NodeType::Root;
      return;
    }
    self.insert_helper(path.as_ref(), full_path, value);
  }

  fn insert_helper(&mut self, mut path: &[u8], full_path: &str, value: V) {
    // Find the longest common prefix.
    // This also implies that the common prefix contains no ':' or '*'
    // since the existing key can't contain those chars.
    let mut i = 0;
    let max = min(path.len(), self.path.len());

    while i < max && path[i] == self.path[i] {
      i += 1;
    }

    // Split edge
    if i < self.path.len() {
      let mut child = Self {
        path: self.path[i..].to_vec(),
        wild_child: self.wild_child,
        indices: self.indices.clone(),
        value: self.value.take(),
        priority: self.priority - 1,
        ..Self::default()
      };

      mem::swap(&mut self.children, &mut child.children);

      self.children = vec![Box::new(child)];
      self.indices = vec![self.path[i]];
      self.path = path[..i].to_vec();
      self.wild_child = false;
      self.value = None;
    }

    // Make new node a child of this node
    if path.len() > i {
      path = &path[i..];

      if self.wild_child {
        return self.children[0].wild_child_conflict(path, full_path, value);
      }

      let idxc = path[0];

      // `/` after param
      if self.node_type == NodeType::Param && idxc == b'/' && self.children.len() == 1 {
        self.children[0].priority += 1;
        return self.children[0].insert_helper(path, full_path, value);
      }

      // Check if a child with the next path byte exists
      for mut i in 0..self.indices.len() {
        if idxc == self.indices[i] {
          i = self.increment_child_prio(i);
          return self.children[i].insert_helper(path, full_path, value);
        }
      }

      // Otherwise insert it
      if idxc != b':' && idxc != b'*' {
        self.indices.push(idxc);

        self.children.push(Box::new(Self::default()));

        let child = self.increment_child_prio(self.indices.len() - 1);
        return self.children[child].insert_child(path, full_path, value);
      }

      self.insert_child(path, full_path, value)
    } else {
      // Otherwise add value to current node
      if self.value.is_some() {
        panic!("a value is already registered for path '{}'", full_path);
      }

      self.value = Some(value);
    }
  }

  #[inline]
  fn wild_child_conflict(&mut self, path: &[u8], full_path: &str, value: V) {
    self.priority += 1;

    // Check if the wildcard matches
    if path.len() >= self.path.len()
      && self.path == &path[..self.path.len()]
      // Adding a child to a CatchAll Node is not possible
      && self.node_type != NodeType::CatchAll
      // Check for longer wildcard, e.g. :name and :names
      && (self.path.len() >= path.len() || path[self.path.len()] == b'/')
    {
      self.insert_helper(path, full_path, value);
    } else {
      // Wildcard conflict
      let path_seg = if self.node_type == NodeType::CatchAll {
        str::from_utf8(path).unwrap()
      } else {
        str::from_utf8(path).unwrap().splitn(2, '/').next().unwrap()
      };

      let prefix = format!(
        "{}{}",
        &full_path[..full_path.find(path_seg).unwrap()],
        str::from_utf8(&self.path).unwrap(),
      );

      panic!(
        "'{}' in new path '{}' conflicts with existing wildcard '{}' in existing prefix '{}'",
        path_seg,
        full_path,
        str::from_utf8(&self.path).unwrap(),
        prefix
      );
    }
  }

  fn insert_child(&mut self, mut path: &[u8], full_path: &str, value: V) {
    let (wildcard, wildcard_index, valid) = find_wildcard(path);

    if wildcard_index.is_none() {
      self.value = Some(value);
      self.path = path.to_vec();
      return;
    };

    let mut wildcard_index = wildcard_index.unwrap();
    let wildcard = wildcard.unwrap();
    // the wildcard name must not contain ':' and '*'
    if !valid {
      panic!(
        "only one wildcard per path segment is allowed, has: '{}' in path '{}'",
        str::from_utf8(wildcard).unwrap(),
        full_path
      );
    };

    // check if the wildcard has a name
    if wildcard.len() < 2 {
      panic!(
        "wildcards must be named with a non-empty name in path '{}'",
        full_path
      );
    }

    // check if this Node existing children which would be
    // unreachable if we insert the wildcard here
    if !self.children.is_empty() {
      panic!(
        "wildcard segment '{}' conflicts with existing children in path '{}'",
        str::from_utf8(wildcard).unwrap(),
        full_path
      )
    }

    // Param
    if wildcard[0] == b':' {
      // Insert prefix before the current wildcard
      if wildcard_index > 0 {
        self.path = path[..wildcard_index].to_vec();
        path = &path[wildcard_index..];
      }

      let child = Self {
        node_type: NodeType::Param,
        path: wildcard.to_vec(),
        ..Self::default()
      };

      self.wild_child = true;
      self.children = vec![Box::new(child)];
      self.children[0].priority += 1;

      // If the path doesn't end with the wildcard, then there
      // will be another non-wildcard subpath starting with '/'

      if wildcard.len() < path.len() {
        path = &path[wildcard.len()..];
        let child = Self {
          priority: 1,
          ..Self::default()
        };

        self.children[0].children = vec![Box::new(child)];
        return self.children[0].children[0].insert_child(path, full_path, value);
      }
      // Otherwise we're done. Insert the value in the new leaf
      self.children[0].value = Some(value);
      return;
    }

    // catch all
    if wildcard_index + wildcard.len() != path.len() {
      panic!(
        "catch-all routes are only allowed at the end of the path in path '{}'",
        full_path
      );
    }

    if !self.path.is_empty() && self.path[self.path.len() - 1] == b'/' {
      panic!(
        "catch-all conflicts with existing value for the path segment root in path '{}'",
        full_path
      );
    }

    // Currently fixed width 1 for '/'
    wildcard_index -= 1;
    if path[wildcard_index] != b'/' {
      panic!("no / before catch-all in path '{}'", full_path);
    }

    // first node: CatchAll Node with empty path
    let child = Self {
      wild_child: true,
      node_type: NodeType::CatchAll,
      ..Self::default()
    };

    self.path = path[..wildcard_index].to_vec();
    self.children = vec![Box::new(child)];
    self.indices = vec![b'/'];
    self.children[0].priority += 1;

    // Second node: node holding the variable
    let child = Self {
      path: path[wildcard_index..].to_vec(),
      node_type: NodeType::CatchAll,
      value: Some(value),
      priority: 1,
      ..Self::default()
    };

    self.children[0].children = vec![Box::new(child)];
  }

  fn match_path(&self, path: &str) -> Result<Match<V>, bool> {
    self.match_helper(path.as_ref(), Params::default())
  }

  // outer loop for walking the tree to get a path's value
  #[inline]
  fn match_helper<'a>(&'a self, mut path: &[u8], params: Params) -> Result<Match<V>, bool> {
    let prefix = self.path.clone();
    if path.len() > prefix.len() {
      if prefix == &path[..prefix.len()] {
        path = &path[prefix.len()..];

        // If this node does not have a wildcard (Param or CatchAll)
        // child, we can just look up the next child node and continue
        // to walk down the tree
        if !self.wild_child {
          let idxc = path[0];
          for i in 0..self.indices.len() {
            if idxc == self.indices[i] {
              return self.children[i].match_helper(path, params);
            }
          }
          // Nothing found.
          // We can recommend to redirect to the same URL without a
          // trailing slash if a leaf exists for that path.
          let tsr = path == [b'/'] && self.value.is_some();
          return Err(tsr);
        }

        return self.children[0].handle_wild_child(path, params);
      }
    } else if path == prefix {
      // We should have reached the node containing the value.
      // Check if this node has a value registered.
      if let Some(value) = self.value.as_ref() {
        return Ok(Match { value, params });
      }

      // If there is no value for this route, but this route has a
      // wildcard child, there must be a value for this path with an
      // additional trailing slash
      if path == [b'/'] && self.wild_child && self.node_type != NodeType::Root {
        return Err(true);
      }

      // No value found. Check if a value for this path + a
      // trailing slash exists for trailing slash recommendation
      for i in 0..self.indices.len() {
        if self.indices[i] == b'/' {
          let tsr = (prefix.len() == 1 && self.children[i].value.is_some())
            || (self.children[i].node_type == NodeType::CatchAll
              && self.children[i].children[0].value.is_some());
          return Err(tsr);
        }
      }

      return Err(false);
    }

    // Nothing found. We can recommend to redirect to the same URL with an
    // extra trailing slash if a leaf exists for that path
    let tsr = (path == [b'/'])
      || (prefix.len() == path.len() + 1
        && prefix[path.len()] == b'/'
        && path == &prefix[..prefix.len() - 1]
        && self.value.is_some());

    Err(tsr)
  }

  // helper function for handling a wildcard child used by `match`
  #[inline]
  fn handle_wild_child<'a>(
    &'a self,
    mut path: &[u8],
    mut params: Params,
  ) -> Result<Match<V>, bool> {
    match self.node_type {
      NodeType::Param => {
        // find param end (either '/' or path end)
        let mut end = 0;
        while end < path.len() && path[end] != b'/' {
          end += 1;
        }

        params.push(Param {
          key: String::from_utf8(self.path[1..].to_vec()).unwrap(),
          value: String::from_utf8(path[..end].to_vec()).unwrap(),
        });

        // we need to go deeper!
        if end < path.len() {
          if !self.children.is_empty() {
            path = &path[end..];

            return self.children[0].match_helper(path, params);
          }

          // ... but we can't
          let tsr = path.len() == end + 1;
          return Err(tsr);
        }

        if let Some(value) = self.value.as_ref() {
          return Ok(Match { value, params });
        } else if self.children.len() == 1 {
          // No value found. Check if a value for this path + a
          // trailing slash exists for TSR recommendation
          let tsr = self.children[0].path == [b'/'] && self.children[0].value.is_some();
          return Err(tsr);
        }

        Err(false)
      }
      NodeType::CatchAll => {
        params.push(Param {
          key: String::from_utf8(self.path[2..].to_vec()).unwrap(),
          value: String::from_utf8(path.to_vec()).unwrap(),
        });

        match self.value.as_ref() {
          Some(value) => Ok(Match { value, params }),
          None => Err(false),
        }
      }
      _ => panic!("invalid node type"),
    }
  }

  fn find_case_insensitive_path(&self, path: &str, fix_trailing_slash: bool) -> Option<String> {
    let mut insensitive_path = Vec::with_capacity(path.len() + 1);
    let found = self.find_case_insensitive_path_helper(
      path.as_bytes(),
      &mut insensitive_path,
      [0; 4],
      fix_trailing_slash,
    );
    if found {
      Some(String::from_utf8(insensitive_path).unwrap())
    } else {
      None
    }
  }

  // recursive case-insensitive match function used by n.find_case_insensitive_path
  fn find_case_insensitive_path_helper(
    &self,
    mut path: &[u8],
    insensitive_path: &mut Vec<u8>,
    mut buf: [u8; 4],
    fix_trailing_slash: bool,
  ) -> bool {
    let lower_path: &[u8] = &path.to_ascii_lowercase();
    if lower_path.len() >= self.path.len()
      && (self.path.is_empty()
        || lower_path[1..self.path.len()].eq_ignore_ascii_case(&self.path[1..]))
    {
      insensitive_path.append(&mut self.path.clone());

      path = &path[self.path.len()..];

      if !path.is_empty() {
        let cached_lower_path = <&[u8]>::clone(&lower_path);

        // If this node does not have a wildcard (param or catchAll) child,
        // we can just look up the next child node and continue to walk down
        // the tree
        if !self.wild_child {
          // skip char bytes already processed
          buf = shift_n_bytes(buf, self.path.len());

          if buf[0] != 0 {
            // old char not finished
            for i in 0..self.indices.len() {
              if self.indices[i] == buf[0] {
                // continue with child node
                return self.children[i].find_case_insensitive_path_helper(
                  path,
                  insensitive_path,
                  buf,
                  fix_trailing_slash,
                );
              }
            }
          } else {
            // process a new char
            let mut current_char = 0 as char;

            // find char start
            // chars are up to 4 byte long,
            // -4 would definitely be another char
            let mut off = 0;
            for j in 0..min(self.path.len(), 3) {
              let i = self.path.len() - j;
              if char_start(cached_lower_path[i]) {
                // read char from cached path
                current_char = str::from_utf8(&cached_lower_path[i..])
                  .unwrap()
                  .chars()
                  .next()
                  .unwrap();
                off = j;
                break;
              }
            }

            current_char.encode_utf8(&mut buf);

            // skip already processed bytes
            buf = shift_n_bytes(buf, off);

            for i in 0..self.indices.len() {
              // lowercase matches
              if self.indices[i] == buf[0] {
                // must use a recursive approach since both the
                // uppercase byte and the lowercase byte might exist
                // as an index
                if self.children[i].find_case_insensitive_path_helper(
                  path,
                  insensitive_path,
                  buf,
                  fix_trailing_slash,
                ) {
                  return true;
                }

                if insensitive_path.len() > self.children[i].path.len() {
                  let prev_len = insensitive_path.len() - self.children[i].path.len();
                  insensitive_path.truncate(prev_len);
                }

                break;
              }
            }

            // same for uppercase char, if it differs
            let up = current_char.to_ascii_uppercase();
            if up != current_char {
              up.encode_utf8(&mut buf);
              buf = shift_n_bytes(buf, off);

              for i in 0..self.indices.len() {
                if self.indices[i] == buf[0] {
                  return self.children[i].find_case_insensitive_path_helper(
                    path,
                    insensitive_path,
                    buf,
                    fix_trailing_slash,
                  );
                }
              }
            }
          }

          // Nothing found. We can recommend to redirect to the same URL
          // without a trailing slash if a leaf exists for that path
          return fix_trailing_slash && path == [b'/'] && self.value.is_some();
        }

        return self.children[0].find_case_insensitive_path_match_helper(
          path,
          insensitive_path,
          buf,
          fix_trailing_slash,
        );
      } else {
        // We should have reached the node containing the value.
        // Check if this node has a value registered.
        if self.value.is_some() {
          return true;
        }

        // No value found.
        // Try to fix the path by adding a trailing slash
        if fix_trailing_slash {
          for i in 0..self.indices.len() {
            if self.indices[i] == b'/' {
              if (self.children[i].path.len() == 1 && self.children[i].value.is_some())
                || (self.children[i].node_type == NodeType::CatchAll
                  && self.children[i].children[0].value.is_some())
              {
                insensitive_path.push(b'/');
                return true;
              }
              return false;
            }
          }
        }
        return false;
      }
    }

    // Nothing found.
    // Try to fix the path by adding / removing a trailing slash
    if fix_trailing_slash {
      if path == [b'/'] {
        return true;
      }
      if lower_path.len() + 1 == self.path.len()
        && self.path[lower_path.len()] == b'/'
        && lower_path[1..].eq_ignore_ascii_case(&self.path[1..lower_path.len()])
        && self.value.is_some()
      {
        insensitive_path.append(&mut self.path.clone());
        return true;
      }
    }

    false
  }

  // recursive case-insensitive match function used by n.findCaseInsensitivePath
  fn find_case_insensitive_path_match_helper(
    &self,
    mut path: &[u8],
    insensitive_path: &mut Vec<u8>,
    buf: [u8; 4],
    fix_trailing_slash: bool,
  ) -> bool {
    match self.node_type {
      NodeType::Param => {
        let mut end = 0;

        while end < path.len() && path[end] != b'/' {
          end += 1;
        }

        let mut path_k = path[..end].to_vec();
        insensitive_path.append(&mut path_k);

        if end < path.len() {
          if !self.children.is_empty() {
            path = &path[end..];

            return self.children[0].find_case_insensitive_path_helper(
              path,
              insensitive_path,
              buf,
              fix_trailing_slash,
            );
          }

          // ... but we can't
          if fix_trailing_slash && path.len() == end + 1 {
            return true;
          }
          return false;
        }

        if self.value.is_some() {
          return true;
        } else if fix_trailing_slash
          && self.children.len() == 1
          && self.children[0].path == [b'/']
          && self.children[0].value.is_some()
        {
          // No value found. Check if a value for this path + a
          // trailing slash exists
          insensitive_path.push(b'/');
          return true;
        }

        false
      }
      NodeType::CatchAll => {
        insensitive_path.append(&mut path.to_vec());
        true
      }
      _ => panic!("invalid node type"),
    }
  }
}

// Shift bytes in array by n bytes left
const fn shift_n_bytes(bytes: [u8; 4], n: usize) -> [u8; 4] {
  match n {
    0 => bytes,
    1 => [bytes[1], bytes[2], bytes[3], 0],
    2 => [bytes[2], bytes[3], 0, 0],
    3 => [bytes[3], 0, 0, 0],
    _ => [0; 4],
  }
}

// This function is ported from go.
// Reports whether the byte could be the first byte of an encoded,
// possibly invalid char. Second and subsequent bytes always have
// the top two bits set to 10.
const fn char_start(b: u8) -> bool {
  b & 0xC0 != 0x80
}

// Search for a wildcard segment and check the name for invalid characters.
fn find_wildcard(path: &[u8]) -> (Option<&[u8]>, Option<usize>, bool) {
  // Find start
  for (start, &c) in path.iter().enumerate() {
    // A wildcard starts with ':' (param) or '*' (catch-all)
    if c != b':' && c != b'*' {
      continue;
    };

    // Find end and check for invalid characters
    let mut valid = true;

    for (end, &c) in path[start + 1..].iter().enumerate() {
      match c {
        b'/' => return (Some(&path[start..start + 1 + end]), Some(start), valid),
        b':' | b'*' => valid = false,
        _ => (),
      };
    }
    return (Some(&path[start..]), Some(start), valid);
  }
  (None, None, false)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::panic;
  use std::sync::Mutex;

  struct TestRequest {
    path: &'static str,
    should_be_nil: bool,
    route: &'static str,
    params: Params,
  }

  impl TestRequest {
    fn new(
      path: &'static str,
      should_be_nil: bool,
      route: &'static str,
      params: Params,
    ) -> TestRequest {
      TestRequest {
        path,
        should_be_nil,
        route,
        params,
      }
    }
  }

  type TestRequests = Vec<TestRequest>;

  fn check_requests<T: Fn() -> String>(tree: &mut Node<T>, requests: TestRequests) {
    for request in requests {
      let res = tree.match_path(request.path);

      match res {
        Err(_) => {
          if !request.should_be_nil {
            panic!("Expected non-nil value for route '{}'", request.path);
          }
        }
        Ok(result) => {
          if request.should_be_nil {
            panic!("Expected nil value for route '{}'", request.path);
          }
          let value = (result.value)();
          if value != request.route {
            panic!(
              "Wrong value for route '{}'. Expected '{}', found '{}')",
              request.path, value, request.route
            );
          }
          assert_eq!(
            result.params, request.params,
            "Wrong params for route '{}'",
            request.path
          );
        }
      };
    }
  }

  fn check_priorities<F: Fn() -> String>(n: &mut Node<F>) -> u32 {
    let mut prio: u32 = 0;
    for i in 0..n.children.len() {
      prio += check_priorities(&mut *n.children[i]);
    }

    if n.value.is_some() {
      prio += 1;
    }

    if n.priority != prio {
      panic!(
        "priority mismatch for node '{}': found '{}', expected '{}'",
        str::from_utf8(&n.path).unwrap(),
        n.priority,
        prio
      )
    }

    prio
  }

  fn fake_value(val: &'static str) -> impl Fn() -> String {
    move || val.to_string()
  }

  #[test]
  fn params() {
    let params = Params(vec![
      Param {
        key: "hello".to_owned(),
        value: "world".to_owned(),
      },
      Param {
        key: "rust-is".to_string(),
        value: "awesome".to_string(),
      },
    ]);

    assert_eq!(params.by_name("hello"), Some("world"));
    assert_eq!(params.by_name("rust-is"), Some("awesome"));
  }

  #[test]
  fn test_tree_add_and_get() {
    let mut tree = Node::default();

    let routes = vec![
      "/hi",
      "/contact",
      "/co",
      "/c",
      "/a",
      "/ab",
      "/doc/",
      "/doc/go_faq.html",
      "/doc/go1.html",
      "/ʯ",
      "/β",
    ];

    for route in routes {
      tree.insert(route, fake_value(route));
    }

    check_requests(
      &mut tree,
      vec![
        TestRequest::new("/a", false, "/a", Params::default()),
        TestRequest::new("/", true, "", Params::default()),
        TestRequest::new("/hi", false, "/hi", Params::default()),
        TestRequest::new("/contact", false, "/contact", Params::default()),
        TestRequest::new("/co", false, "/co", Params::default()),
        TestRequest::new("/con", true, "", Params::default()), // key mismatch
        TestRequest::new("/cona", true, "", Params::default()), // key mismatch
        TestRequest::new("/no", true, "", Params::default()),  // no matching child
        TestRequest::new("/ab", false, "/ab", Params::default()),
        TestRequest::new("/ʯ", false, "/ʯ", Params::default()),
        TestRequest::new("/β", false, "/β", Params::default()),
      ],
    );

    check_priorities(&mut tree);
  }

  #[test]
  fn test_tree_wildcard() {
    let mut tree = Node::default();

    let routes = vec![
      "/",
      "/cmd/:tool/:sub",
      "/cmd/:tool/",
      "/src/*filepath",
      "/search/",
      "/search/:query",
      "/user_:name",
      "/user_:name/about",
      "/files/:dir/*filepath",
      "/doc/",
      "/doc/go_faq.html",
      "/doc/go1.html",
      "/info/:user/public",
      "/info/:user/project/:project",
    ];

    for route in routes {
      tree.insert(route, fake_value(route));
    }

    check_requests(
      &mut tree,
      vec![
        TestRequest::new("/", false, "/", Params::default()),
        TestRequest::new(
          "/cmd/test/",
          false,
          "/cmd/:tool/",
          Params(vec![Param::new("tool", "test")]),
        ),
        TestRequest::new(
          "/cmd/test",
          true,
          "",
          Params(vec![Param::new("tool", "test")]),
        ),
        TestRequest::new(
          "/cmd/test/3",
          false,
          "/cmd/:tool/:sub",
          Params(vec![Param::new("tool", "test"), Param::new("sub", "3")]),
        ),
        TestRequest::new(
          "/src/",
          false,
          "/src/*filepath",
          Params(vec![Param::new("filepath", "/")]),
        ),
        TestRequest::new(
          "/src/some/file.png",
          false,
          "/src/*filepath",
          Params(vec![Param::new("filepath", "/some/file.png")]),
        ),
        TestRequest::new("/search/", false, "/search/", Params::default()),
        TestRequest::new(
          "/search/someth!ng+in+ünìcodé",
          false,
          "/search/:query",
          Params(vec![Param::new("query", "someth!ng+in+ünìcodé")]),
        ),
        TestRequest::new(
          "/search/someth!ng+in+ünìcodé/",
          true,
          "",
          Params(vec![Param::new("query", "someth!ng+in+ünìcodé")]),
        ),
        TestRequest::new(
          "/user_rustacean",
          false,
          "/user_:name",
          Params(vec![Param::new("name", "rustacean")]),
        ),
        TestRequest::new(
          "/user_rustacean/about",
          false,
          "/user_:name/about",
          Params(vec![Param::new("name", "rustacean")]),
        ),
        TestRequest::new(
          "/files/js/inc/framework.js",
          false,
          "/files/:dir/*filepath",
          Params(vec![
            Param::new("dir", "js"),
            Param::new("filepath", "/inc/framework.js"),
          ]),
        ),
        TestRequest::new(
          "/info/gordon/public",
          false,
          "/info/:user/public",
          Params(vec![Param::new("user", "gordon")]),
        ),
        TestRequest::new(
          "/info/gordon/project/go",
          false,
          "/info/:user/project/:project",
          Params(vec![
            Param::new("user", "gordon"),
            Param::new("project", "go"),
          ]),
        ),
      ],
    );

    check_priorities(&mut tree);
  }

  type TestRoute = (&'static str, bool);

  fn test_routes(routes: Vec<TestRoute>) {
    let tree = Mutex::new(Node::default());

    for route in routes {
      let recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route.0, ());
      });

      if route.1 {
        if recv.is_ok() {
          // panic!("no panic for conflicting route '{}'", route.0);
        }
      } else if recv.is_err() {
        panic!("unexpected panic for route '{}': {:?}", route.0, recv);
      }
    }
  }

  #[test]
  fn test_tree_wildcard_conflict() {
    let routes = vec![
      ("/cmd/:tool/:sub", false),
      ("/cmd/vet", true),
      ("/src/*filepath", false),
      ("/src/*filepathx", true),
      ("/src/", true),
      ("/src1/", false),
      ("/src1/*filepath", true),
      ("/src2*filepath", true),
      ("/search/:query", false),
      ("/search/invalid", true),
      ("/user_:name", false),
      ("/user_x", true),
      ("/user_:name", true),
      ("/id:id", false),
      ("/id/:id", true),
    ];
    test_routes(routes);
  }

  #[test]
  fn test_tree_child_conflict() {
    let routes = vec![
      ("/cmd/vet", false),
      ("/cmd/:tool/:sub", true),
      ("/src/AUTHORS", false),
      ("/src/*filepath", true),
      ("/user_x", false),
      ("/user_:name", true),
      ("/id/:id", false),
      ("/id:id", true),
      ("/:id", true),
      ("/*filepath", true),
    ];

    test_routes(routes);
  }

  #[test]
  fn test_tree_duplicate_path() {
    let tree = Mutex::new(Node::default());

    let routes = vec![
      "/",
      "/doc/",
      "/src/*filepath",
      "/search/:query",
      "/user_:name",
    ];

    for route in routes {
      let mut recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route, fake_value(route));
      });

      if recv.is_err() {
        panic!("panic inserting route '{}': {:?}", route, recv);
      }

      recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route, fake_value(route));
      });

      if recv.is_ok() {
        panic!("no panic while inserting duplicate route '{}'", route);
      }
    }

    check_requests(
      &mut tree.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
      vec![
        TestRequest::new("/", false, "/", Params::default()),
        TestRequest::new("/doc/", false, "/doc/", Params::default()),
        TestRequest::new(
          "/src/some/file.png",
          false,
          "/src/*filepath",
          Params(vec![Param::new("filepath", "/some/file.png")]),
        ),
        TestRequest::new(
          "/search/someth!ng+in+ünìcodé",
          false,
          "/search/:query",
          Params(vec![Param::new("query", "someth!ng+in+ünìcodé")]),
        ),
        TestRequest::new(
          "/user_rustacean",
          false,
          "/user_:name",
          Params(vec![Param::new("name", "rustacean")]),
        ),
      ],
    );
  }

  #[test]
  fn test_empty_wildcard_name() {
    let tree = Mutex::new(Node::default());
    let routes = vec!["/user:", "/user:/", "/cmd/:/", "/src/*"];

    for route in routes {
      let recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route, fake_value(route));
      });

      if recv.is_ok() {
        panic!(
          "no panic while inserting route with empty wildcard name '{}",
          route
        );
      }
    }
  }

  #[test]
  fn test_tree_catch_all_conflict() {
    let routes = vec![
      ("/src/*filepath/x", true),
      ("/src2/", false),
      ("/src2/*filepath/x", true),
    ];

    test_routes(routes);
  }

  #[test]
  fn test_tree_catch_all_conflict_root() {
    let routes = vec![("/", false), ("/*filepath", true)];

    test_routes(routes);
  }

  #[test]
  fn test_tree_double_wildcard() {
    let panic_msg = "only one wildcard per path segment is allowed";
    let routes = vec!["/:foo:bar", "/:foo:bar/", "/:foo*bar"];

    for route in routes {
      let tree = Mutex::new(Node::default());
      let recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route, fake_value(route));
      });

      if recv.is_ok() {
        panic!(panic_msg);
      }
    }
  }

  #[test]
  fn test_tree_trailing_slash_redirect() {
    let tree = Mutex::new(Node::default());
    let routes = vec![
      "/hi",
      "/b/",
      "/search/:query",
      "/cmd/:tool/",
      "/src/*filepath",
      "/x",
      "/x/y",
      "/y/",
      "/y/z",
      "/0/:id",
      "/0/:id/1",
      "/1/:id/",
      "/1/:id/2",
      "/aa",
      "/a/",
      "/admin",
      "/admin/:category",
      "/admin/:category/:page",
      "/doc",
      "/doc/go_faq.html",
      "/doc/go1.html",
      "/no/a",
      "/no/b",
      "/api/hello/:name",
    ];

    for route in routes {
      let recv = panic::catch_unwind(|| {
        let mut guard = match tree.lock() {
          Ok(guard) => guard,
          Err(poisoned) => poisoned.into_inner(),
        };
        guard.insert(route, fake_value(route));
      });

      if recv.is_err() {
        panic!("panic inserting route '{}': {:?}", route, recv);
      }
    }

    let tsr_routes = vec![
      "/hi/",
      "/b",
      "/search/rustacean/",
      "/cmd/vet",
      "/src",
      "/x/",
      "/y",
      "/0/go/",
      "/1/go",
      "/a",
      "/admin/",
      "/admin/config/",
      "/admin/config/permissions/",
      "/doc/",
    ];

    for route in tsr_routes {
      let guard = match tree.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
      };
      let res = guard.match_path(route);

      match res {
        Ok(_) => {
          panic!("non-nil value for TSR route '{}'", route);
        }
        Err(tsr) => {
          if !tsr {
            panic!("expected TSR recommendation for route '{}'", route);
          }
        }
      }
    }

    let no_tsr_routes = vec!["/", "/no", "/no/", "/_", "/_/", "/api/world/abc"];

    for route in no_tsr_routes {
      let guard = match tree.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
      };
      let res = guard.match_path(route);

      match res {
        Ok(_) => {
          panic!("non-nil value for TSR route '{}'", route);
        }
        Err(tsr) => {
          if tsr {
            panic!("expected no TSR recommendation for route '{}'", route);
          }
        }
      }
    }
  }

  #[test]
  fn test_tree_root_trailing_slash_redirect() {
    let mut tree = Node::default();

    tree.insert("/:test", fake_value("/:test"));

    let res = tree.match_path("/");

    match res {
      Ok(_) => {
        panic!("non-nil value for route '/'");
      }
      Err(tsr) => {
        if tsr {
          panic!("expected no TSR recommendation for route '/'");
        }
      }
    }
  }

  #[test]
  fn test_tree_find_case_insensitive_path() {
    let mut tree = Node::default();

    let routes = vec![
      "/hi",
      "/b/",
      "/ABC/",
      "/search/:query",
      "/cmd/:tool/",
      "/src/*filepath",
      "/x",
      "/x/y",
      "/y/",
      "/y/z",
      "/0/:id",
      "/0/:id/1",
      "/1/:id/",
      "/1/:id/2",
      "/aa",
      "/a/",
      "/doc",
      "/doc/go_faq.html",
      "/doc/go1.html",
      "/doc/go/away",
      "/no/a",
      "/no/b",
      "/Π",
      "/u/apfêl/",
      "/u/äpfêl/",
      "/u/öpfêl",
      "/v/Äpfêl/",
      "/v/Öpfêl",
      "/w/♬",
      "/w/♭/",
      "/w/𠜎",
      "/w/𠜏/",
      "/loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
    ];

    for route in &routes {
      tree.insert(route, fake_value(route));
    }

    // Check out == in for all registered routes
    // With fixTrailingSlash = true
    for route in &routes {
      let out = tree.find_case_insensitive_path(route, true);
      match out {
        None => panic!("Route '{}' not found!", route),
        Some(out) => {
          if out != *route {
            panic!("Wrong result for route '{}': {}", route, out);
          }
        }
      };
    }

    // With fixTrailingSlash = false
    for route in &routes {
      let out = tree.find_case_insensitive_path(route, false);
      match out {
        None => panic!("Route '{}' not found!", route),
        Some(out) => {
          if out != *route {
            panic!("Wrong result for route '{}': {}", route, out);
          }
        }
      };
    }

    let tests = vec![
      ("/HI", "/hi", false),
      ("/HI/", "/hi", true),
      ("/B", "/b/", true),
      ("/B/", "/b/", false),
      ("/abc", "/ABC/", true),
      ("/abc/", "/ABC/", false),
      ("/aBc", "/ABC/", true),
      ("/aBc/", "/ABC/", false),
      ("/abC", "/ABC/", true),
      ("/abC/", "/ABC/", false),
      ("/SEARCH/QUERY", "/search/QUERY", false),
      ("/SEARCH/QUERY/", "/search/QUERY", true),
      ("/CMD/TOOL/", "/cmd/TOOL/", false),
      ("/CMD/TOOL", "/cmd/TOOL/", true),
      ("/SRC/FILE/PATH", "/src/FILE/PATH", false),
      ("/x/Y", "/x/y", false),
      ("/x/Y/", "/x/y", true),
      ("/X/y", "/x/y", false),
      ("/X/y/", "/x/y", true),
      ("/X/Y", "/x/y", false),
      ("/X/Y/", "/x/y", true),
      ("/Y/", "/y/", false),
      ("/Y", "/y/", true),
      ("/Y/z", "/y/z", false),
      ("/Y/z/", "/y/z", true),
      ("/Y/Z", "/y/z", false),
      ("/Y/Z/", "/y/z", true),
      ("/y/Z", "/y/z", false),
      ("/y/Z/", "/y/z", true),
      ("/Aa", "/aa", false),
      ("/Aa/", "/aa", true),
      ("/AA", "/aa", false),
      ("/AA/", "/aa", true),
      ("/aA", "/aa", false),
      ("/aA/", "/aa", true),
      ("/A/", "/a/", false),
      ("/A", "/a/", true),
      ("/DOC", "/doc", false),
      ("/DOC/", "/doc", true),
      ("/NO", "", true),
      ("/DOC/GO", "", true),
      // [TODO] unicode vs ascii case sensitivity
      // ("/π", "/Π", false)
      // ("/π/", "/Π", true),
      // ("/u/ÄPFÊL/", "/u/äpfêl/", false)
      // ("/u/ÄPFÊL", "/u/äpfêl/", true),
      // ("/u/ÖPFÊL/", "/u/öpfêl", true),
      // ("/u/ÖPFÊL", "/u/öpfêl", false)
      // ("/v/äpfêL/", "/v/Äpfêl/", false)
      // ("/v/äpfêL", "/v/Äpfêl/", true),
      // ("/v/öpfêL/", "/v/Öpfêl", true),
      // ("/v/öpfêL", "/v/Öpfêl", false)
      ("/w/♬/", "/w/♬", true),
      ("/w/♭", "/w/♭/", true),
      ("/w/𠜎/", "/w/𠜎", true),
      ("/w/𠜏", "/w/𠜏/", true),
      (
        "/lOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOng/",
        "/loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong",
        true),
    ];

    struct Test {
      inn: &'static str,
      out: &'static str,
      slash: bool,
    };

    let tests: Vec<Test> = tests
      .into_iter()
      .map(|test| Test {
        inn: test.0,
        out: test.1,
        slash: test.2,
      })
      .collect();

    // With fixTrailingSlash = true
    for test in &tests {
      let res = tree.find_case_insensitive_path(test.inn, true);
      match res {
        None => (),
        Some(res) => {
          if res != test.out {
            panic!("Wrong result for route '{}': {}", res, test.out);
          }
        }
      };
    }

    // With fixTrailingSlash = false
    for test in &tests {
      let res = tree.find_case_insensitive_path(test.inn, false);
      match res {
        None => (),
        Some(res) => {
          if test.slash {
            // test needs a trailingSlash fix. It must not be found!
            panic!("Found without fixTrailingSlash: {}; got {}", test.inn, res);
          }
          if res != test.out {
            panic!("Wrong result for route '{}': {}", res, test.out);
          }
        }
      };
    }
  }

  #[test]
  #[should_panic(expected = "conflicts with existing wildcard")]
  fn test_tree_wildcard_conflict_ex() {
    let conflicts = vec![
      "/who/are/foo",
      "/who/are/foo/",
      "/who/are/foo/bar",
      "/conxxx",
      "xxx",
      "/conooo/xxx",
    ];

    for conflict in conflicts {
      // I have to re-create a 'tree', because the 'tree' will be
      // in an inconsistent state when the loop recovers from the
      // panic which threw by 'addRoute' function.
      let mut tree = Node::default();

      let routes = vec!["/con:tact", "/who/are/*you", "/who/foo/hello"];

      for route in routes {
        tree.insert(route, fake_value(route));
      }
      tree.insert(conflict, fake_value(conflict));
    }
  }
}

