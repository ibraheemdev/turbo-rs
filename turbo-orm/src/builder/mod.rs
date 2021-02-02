mod column;
mod foreign;
mod ops;
mod reference;

pub use column::ColumnBuilder;
pub use foreign::ForeignKeyBuilder;
pub use ops::Op;
pub use reference::ReferenceBuilder;

/// Types that can be converted into SQL.
pub trait Builder {
  /// Returns the sql representation of the element
  /// and its arguments (if any).
  fn build(self) -> (String, Vec<String>);
}

/// An extension trait for query builders that provides a variety of convenient functions.
pub trait BuilderExt {
  /// Quotes the given identifier with the characters based
  /// on the configured dialect. It defaults to "`".
  fn quote(&self, ident: impl Into<String>) -> String;

  /// Appends the given string as an identifier.
  fn ident(self, s: impl AsRef<str>) -> Self;

  /// Calls [`ident`](self::BaseBuilder::ident) on all arguments and adds a comma between them.
  fn ident_comma<I, S>(self, s: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>;

  /// Push an operator to the builder.
  fn op(self, op: Op) -> Self;

  /// Append an input argument to the builder.
  fn arg(self, arg: impl Arg) -> Self;

  /// Join this builder with an existing builder.
  fn join(self, builder: impl Builder) -> Self;

  /// Accepts a callback, and wraps its result with parentheses.
  fn nested(self, f: impl FnMut(BaseBuilder)) -> Self;

  /// Push a raw string to the builder.
  fn push_str(self, s: impl AsRef<str>) -> Self;

  /// Push a raw char to the builder.
  fn push(self, c: char) -> Self;

  /// Add a comma to the query.
  fn comma(self) -> Self;

  /// Add a space to the query.
  fn pad(self) -> Self;

  /// Whether the given string is a dialect identifier.
  fn is_ident(&self, s: impl AsRef<str>) -> bool;

  /// Returns the dialect of the builder.
  fn dialect(&self) -> &Dialect;

  /// Sets the builder dialect. It's used for garnering dialect specific queries.
  fn set_dialect(&mut self, dialect: Dialect) -> &Self;

  /// Returns the total number of arguments so far.
  fn total(&self) -> usize;

  /// Sets the value of the total arguments.
  /// Used to pass this information between sub builders/expressions.
  fn set_total(&mut self, total: usize) -> &Self;
}

/// The base type used by more specific query builders such as `DeleteBuilder` and `ColumnBuilder`. You generally will not have to interact with this type directly.
#[derive(Default)]
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

/// The supported SQL dialects.
#[derive(Clone)]
pub enum Dialect {
  MySQL,
  SQLite,
  Postgres,
}

impl Default for Dialect {
  fn default() -> Self {
    Dialect::Postgres
  }
}

impl BuilderExt for BaseBuilder {
  #[inline]
  fn quote(&self, ident: impl Into<String>) -> String {
    let ident = ident.into();
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

  fn ident(mut self, s: impl AsRef<str>) -> Self {
    let s = s.as_ref();
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
  fn ident_comma<I, S>(self, s: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    for (i, s) in s.into_iter().enumerate() {
      if i > 0 {
        self = self.comma();
      }
      self = self.ident(s);
    }
    self
  }

  #[inline]
  fn op(self, op: Op) -> Self {
    match op {
      Op::IsNull | Op::NotNull => self.pad().push_str(op.as_str()),
      _ => self.pad().push_str(op.as_str()).pad(),
    }
  }

  #[inline]
  fn arg(self, arg: impl Arg) -> Self {
    arg.append(self)
  }

  fn join(mut self, builder: impl Builder) -> Self {
    let (query, mut args) = builder.build();
    self.args.append(&mut args);
    self.total = self.args.len();
    self.push_str(&query)
  }

  #[inline]
  fn nested(mut self, f: impl FnMut(Self)) -> Self {
    self = self.push('(');
    f(self);
    self.push(')')
  }

  #[inline]
  fn push_str(mut self, s: impl AsRef<str>) -> Self {
    self.buf.push_str(s.as_ref());
    self
  }

  #[inline]
  fn push(mut self, c: char) -> Self {
    self.buf.push(c);
    self
  }

  #[inline]
  fn comma(self) -> Self {
    self.push_str(", ")
  }

  #[inline]
  fn pad(self) -> Self {
    self.push_str(" ")
  }

  #[inline]
  fn is_ident(&self, s: impl AsRef<str>) -> bool {
    match self.dialect {
      Dialect::Postgres => s.as_ref().contains("\""),
      _ => s.as_ref().contains("`"),
    }
  }

  #[inline]
  fn dialect(&self) -> &Dialect {
    &self.dialect
  }

  #[inline]
  fn set_dialect(&mut self, dialect: Dialect) -> &Self {
    self.dialect = dialect;
    self
  }

  #[inline]
  fn total(&self) -> usize {
    self.total
  }

  #[inline]
  fn set_total(&mut self, total: usize) -> &Self {
    self.total = total;
    self
  }
}

trait ChildBuilder {
  fn parent(&mut self) -> &mut BaseBuilder;
}

impl<T> BuilderExt for T
where
  T: ChildBuilder,
{
  #[inline]
  fn quote(&self, ident: impl Into<String>) -> String {
    self.parent().quote(ident)
  }

  #[inline]
  fn ident(self, s: impl AsRef<str>) -> Self {
    self.parent().ident(s);
    self
  }

  #[inline]
  fn ident_comma<I, S>(self, s: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    self.parent().ident_comma(s);
    self
  }

  #[inline]
  fn op(self, op: Op) -> Self {
    self.parent().op(op);
    self
  }

  #[inline]
  fn arg(self, arg: impl Arg) -> Self {
    self.parent().arg(arg);
    self
  }

  #[inline]
  fn join(self, builder: impl Builder) -> Self {
    self.parent().join(builder);
    self
  }

  #[inline]
  fn nested(self, f: impl FnMut(BaseBuilder)) -> Self {
    self.parent().nested(f);
    self
  }

  #[inline]
  fn push_str(self, s: impl AsRef<str>) -> Self {
    self.parent().push_str(s);
    self
  }

  #[inline]
  fn push(self, c: char) -> Self {
    self.parent().push(c);
    self
  }

  #[inline]
  fn comma(self) -> Self {
    self.parent().comma();
    self
  }

  #[inline]
  fn pad(self) -> Self {
    self.parent().pad();
    self
  }

  #[inline]
  fn is_ident(&self, s: impl AsRef<str>) -> bool {
    self.parent().is_ident(s)
  }

  #[inline]
  fn dialect(&self) -> &Dialect {
    self.parent().dialect()
  }

  #[inline]
  fn set_dialect(&mut self, dialect: Dialect) -> &Self {
    self.parent().set_dialect(dialect);
    self
  }

  #[inline]
  fn total(&self) -> usize {
    self.parent().total()
  }

  #[inline]
  fn set_total(&mut self, total: usize) -> &Self {
    self.parent().set_total(total);
    self
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
