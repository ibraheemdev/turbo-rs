use regex::Regex;

pub struct RegexGroup {
  host: RouteRegex,
  path: RouteRegex,
  queries: Vec<RouteRegex>,
}

pub struct RouteRegex {
  // The unmodified template.
  template: String,
  // The type of match
  regex_type: RegexType,
  // Options for matching
  options: RegexOptions,
  // Expanded regexp.
  regexp: Regex,
  // Reverse template.
  reverse: String,
  // Variable names.
  var_names: Vec<String>,
  // Variable regexps (validators).
  var_regexs: Vec<Regex>,
  // Wildcard host-port (no strict port match in hostname)
  wildcardHostPort: bool,
}

pub enum RegexType {
  Path,
  Host,
  Prefix,
  Query,
}

pub struct RegexOptions {
  strict_slash: bool,
  use_encoded_path: bool,
}
