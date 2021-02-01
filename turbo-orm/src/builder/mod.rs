mod column;
mod foreign;
mod ops;

use ops::Op;

pub trait Builder {
  // Returns the sql representation of the element
  // and its arguments (if any).
  fn build(self) -> (String, Vec<String>);
}

pub struct BaseBuilder {
  // underlying buffer
  buf: String,
  // query parameters
  args: Vec<String>,
  // total parameters in the query tree
  total: usize,
  // configured dialect
  dialect: Dialect,
}

#[derive(Clone)]
pub enum Dialect {
  MySQL,
  SQLite,
  Postgres,
}

impl BaseBuilder {
  // Quotes the given identifier with the characters based
  // on the configured dialect. It defaults to "`".
  #[inline]
  pub fn quote(&self, ident: String) -> String {
    match self.dialect {
      Dialect::Postgres => {
        if ident.contains("`") {
          return ident.replace("`", "\"");
        }
        ident
      }
      _ => format!("`{}`", ident),
    }
  }

  // Appends the given string as an identifier.
  pub fn ident(mut self, s: &str) -> Self {
    if s.len() == 0 {
    } else if s != "*" && !self.is_ident(s) && !is_func(s) & !is_mod(s) {
      self = self.push_str(s);
    } else if is_func(s) || is_mod(s) {
      // Modifiers and aggregation functions that
      // were called without dialect information.
      self = self.push_str(&s.replace("`", "\""));
    } else {
      self = self.push_str(s);
    }

    self
  }

  #[inline]

  // Calls [`ident`](self::BaseBuilder::ident) on all arguments and adds a comma between them.
  pub fn ident_comma(mut self, s: &[&str]) -> Self {
    for i in 0..s.len() {
      if i > 0 {
        self = self.comma();
      }
      self = self.ident(s[i]);
    }
    self
  }

  // Push an operator to the builder.
  #[inline]
  pub fn op(self, op: Op) -> Self {
    match op {
      Op::IsNull | Op::NotNull => self.pad().push_str(op.as_str()),
      _ => self.pad().push_str(op.as_str()).pad(),
    }
  }

  // Append an input argument to the builder.
  #[inline]
  pub fn arg(self, arg: impl Arg) -> Self {
    arg.append(self)
  }

  // Join this builder with an existing builder.
  pub fn join(mut self, builder: impl Builder + BuilderState) -> Self {
    let (query, mut args) = builder.build();
    self.args.append(&mut args);
    self.total = self.args.len();
    self.push_str(&query)
  }

  // Accepts a callback, and wraps its result with parentheses.
  pub fn nested(mut self, f: impl Fn(Self) -> Self) -> Self {
    let mut nb = Self {
      dialect: self.dialect.clone(),
      total: self.total,
      buf: String::new(),
      args: Vec::new(),
    };
    nb = nb.push('(');
    nb = f(nb);
    self.args.append(&mut nb.args);
    self.total = nb.total;
    self.push_str(&nb.buf).push(')')
  }

  // Push a raw string to the builder.
  #[inline]
  pub fn push_str(mut self, s: &str) -> Self {
    self.buf.push_str(s);
    self
  }

  // Push a raw char to the builder.
  #[inline]
  pub fn push(mut self, c: char) -> Self {
    self.buf.push(c);
    self
  }

  // Add a comma to the query.
  #[inline]
  pub fn comma(self) -> Self {
    self.push_str(", ")
  }

  // Add a space to the query.
  #[inline]
  pub fn pad(self) -> Self {
    self.push_str(" ")
  }

  // Whether the given string is a dialect identifier.
  #[inline]
  pub fn is_ident(&self, s: &str) -> bool {
    match self.dialect {
      Dialect::Postgres => s.contains("\""),
      _ => s.contains("`"),
    }
  }
}

pub trait Arg {
  fn append(self, builder: BaseBuilder) -> BaseBuilder;
}

pub struct Raw<'a>(&'a str);

impl Arg for Raw<'_> {
  #[inline]
  fn append(self, builder: BaseBuilder) -> BaseBuilder {
    builder.push_str(self.0)
  }
}

impl<T: ToString> Arg for T {
  fn append(self, mut builder: BaseBuilder) -> BaseBuilder {
    builder.total += 1;
    builder.args.push(self.to_string());
    match builder.dialect {
      Dialect::Postgres => {
        let total = builder.total.to_string();
        builder = builder.push_str("$").push_str(&total)
      }
      _ => builder = builder.push_str("?"),
    };
    builder
  }
}

#[inline]
fn is_func(s: &str) -> bool {
  s.contains("(") && s.contains(")")
}

const MODS: [&str; 3] = ["DISTINCT", "ALL", "WITH ROLLUP"];

#[inline]
fn is_mod(s: &str) -> bool {
  for i in 0..2 {
    if s.starts_with(MODS[i]) {
      return true;
    }
  }
  false
}

pub trait BuilderState {
  // Returns the dialect of the builder.
  fn dialect(&self) -> &Dialect;
  // Sets the builder dialect. It's used for garnering dialect specific queries.
  fn set_dialect(&mut self, dialect: Dialect) -> &Self;
  // Returns the total number of arguments so far.
  fn total(&self) -> usize;
  // Sets the value of the total arguments.
  // Used to pass this information between sub builders/expressions.
  fn set_total(&mut self, total: usize) -> &Self;
}

impl BuilderState for BaseBuilder {
  fn dialect(&self) -> &Dialect {
    &self.dialect
  }

  fn set_dialect(&mut self, dialect: Dialect) -> &Self {
    self.dialect = dialect;
    self
  }

  fn total(&self) -> usize {
    self.total
  }

  fn set_total(&mut self, total: usize) -> &Self {
    self.total = total;
    self
  }
}
