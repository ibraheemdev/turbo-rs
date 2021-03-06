mod alter_index;
mod alter_table;
mod column;
mod drop_index;
mod foreign;
mod index;
mod insert;
mod ops;
mod raw;
mod reference;
mod table;

pub use alter_index::AlterIndexBuilder;
pub use alter_table::AlterTableBuilder;
pub use column::ColumnBuilder;
pub use drop_index::DropIndexBuilder;
pub use foreign::ForeignKeyBuilder;
pub use index::IndexBuilder;
pub use insert::InsertBuilder;
pub use ops::Op;
pub use raw::Raw;
pub use reference::ReferenceBuilder;
pub use table::TableBuilder;

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

  fn schema(&mut self, schema: impl AsRef<str>) -> &mut Self;

  /// Appends the given string as an identifier.
  fn ident(&mut self, s: impl AsRef<str>) -> &mut Self;

  /// Calls [`ident`](self::BaseBuilder::ident) on all arguments and adds a comma between them.
  fn ident_comma<I, S>(&mut self, s: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>;

  /// Push an operator to the builder.
  fn op(&mut self, op: Op) -> &mut Self;

  /// Append an input argument to the builder.
  fn arg(&mut self, arg: impl ToString) -> &mut Self;

  /// Join this builder with an existing builder.
  fn join(&mut self, builder: impl Builder) -> &mut Self;

  /// Join this builder with a list of comma seperated builders.
  fn join_many<I, B>(&mut self, builders: I) -> &mut Self
  where
    I: IntoIterator<Item = B>,
    B: Builder;

  /// Accepts a callback, and wraps its result with parentheses.
  fn nested(&mut self, f: impl FnOnce(&mut BaseBuilder)) -> &mut Self;

  /// Push a raw string to the builder.
  fn push_str(&mut self, s: impl AsRef<str>) -> &mut Self;

  /// Push a raw char to the builder.
  fn push(&mut self, c: char) -> &mut Self;

  /// Prefix the builder query with a raw string.
  fn prefix(&mut self, s: impl AsRef<str>) -> &mut Self;

  /// Add a comma to the query.
  fn comma(&mut self) -> &mut Self;

  /// Add a space to the query.
  fn pad(&mut self) -> &mut Self;

  /// Whether the given string is a dialect identifier.
  fn is_ident(&self, s: impl AsRef<str>) -> bool;

  /// Returns the dialect of the builder.
  fn dialect(&self) -> &Dialect;

  /// Sets the builder dialect. It's used for garnering dialect specific queries.
  fn set_dialect(&mut self, dialect: Dialect) -> &mut Self;

  /// Returns the total number of arguments so far.
  fn total(&self) -> usize;

  /// Sets the value of the total arguments.
  /// Used to pass this information between sub builders/expressions.
  fn set_total(&mut self, total: usize) -> &mut Self;
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
#[derive(Clone, PartialEq, Eq)]
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

impl Builder for BaseBuilder {
  #[inline]
  fn build(self) -> (String, Vec<String>) {
    (self.buf, self.args)
  }
}

impl BaseBuilder {
  fn join_helper<I, B>(&mut self, builders: I, seperator: impl AsRef<str>) -> &mut Self
  where
    I: IntoIterator<Item = B>,
    B: Builder,
  {
    for (i, b) in builders.into_iter().enumerate() {
      if i > 0 {
        self.push_str(seperator.as_ref());
      }
      let (query, mut args) = b.build();
      self.push_str(query);
      self.total = args.len();
      self.args.append(&mut args);
    }
    self
  }
}

impl BuilderExt for BaseBuilder {
  #[inline]
  fn quote(&self, ident: impl Into<String>) -> String {
    let ident = ident.into();
    match self.dialect {
      Dialect::Postgres => {
        if ident.contains('`') {
          return ident.replace("`", "\"");
        }
        ident
      }
      _ => format!("`{}`", ident),
    }
  }

  fn schema(&mut self, schema: impl AsRef<str>) -> &mut Self {
    let schema = schema.as_ref();
    if !schema.is_empty() && self.dialect != Dialect::SQLite {
      self.ident(schema).push('.');
    }
    self
  }

  fn ident(&mut self, s: impl AsRef<str>) -> &mut Self {
    let s = s.as_ref();
    if s.is_empty() {
    } else if s != "*" && !self.is_ident(s) && !is_func(s) & !is_mod(s) {
      self.push_str(s);
    } else if is_func(s) || is_mod(s) {
      // Modifiers and aggregation functions that
      // were called without dialect information.
      self.push_str(&s.replace("`", "\""));
    } else {
      self.push_str(s);
    }

    self
  }

  #[inline]
  fn ident_comma<I, S>(&mut self, s: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    for (i, s) in s.into_iter().enumerate() {
      if i > 0 {
        self.comma();
      }
      self.ident(s);
    }
    self
  }

  #[inline]
  fn op(&mut self, op: Op) -> &mut Self {
    match op {
      Op::IsNull | Op::NotNull => self.pad().push_str(op.as_str()),
      _ => self.pad().push_str(op.as_str()).pad(),
    }
  }

  #[inline]
  fn arg(&mut self, arg: impl ToString) -> &mut Self {
    self.total += 1;
    self.args.push(arg.to_string());
    match self.dialect {
      Dialect::Postgres => {
        let total = self.total.to_string();
        self.push_str("$").push_str(&total);
      }
      _ => {
        self.push_str("?");
      }
    };
    self
  }

  #[inline]
  fn join(&mut self, builder: impl Builder) -> &mut Self {
    self.join_helper(std::iter::once(builder), "")
  }

  #[inline]
  fn join_many<I, B>(&mut self, builders: I) -> &mut Self
  where
    I: IntoIterator<Item = B>,
    B: Builder,
  {
    self.join_helper(builders, ", ")
  }

  #[inline]
  fn nested(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
    self.push('(');
    f(self);
    self.push(')')
  }

  #[inline]
  fn push_str(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.buf.push_str(s.as_ref());
    self
  }

  #[inline]
  fn push(&mut self, c: char) -> &mut Self {
    self.buf.push(c);
    self
  }

  #[inline]
  fn prefix(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.buf.insert_str(0, s.as_ref());
    self
  }

  #[inline]
  fn comma(&mut self) -> &mut Self {
    self.push_str(", ")
  }

  #[inline]
  fn pad(&mut self) -> &mut Self {
    self.push_str(" ")
  }

  #[inline]
  fn is_ident(&self, s: impl AsRef<str>) -> bool {
    match self.dialect {
      Dialect::Postgres => s.as_ref().contains('"'),
      _ => s.as_ref().contains('`'),
    }
  }

  #[inline]
  fn dialect(&self) -> &Dialect {
    &self.dialect
  }

  #[inline]
  fn set_dialect(&mut self, dialect: Dialect) -> &mut Self {
    self.dialect = dialect;
    self
  }

  #[inline]
  fn total(&self) -> usize {
    self.total
  }

  #[inline]
  fn set_total(&mut self, total: usize) -> &mut Self {
    self.total = total;
    self
  }
}

#[doc(hidden)]
pub trait ChildBuilder {
  fn parent(&self) -> &BaseBuilder;
  fn parent_mut(&mut self) -> &mut BaseBuilder;
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
  fn schema(&mut self, schema: impl AsRef<str>) -> &mut Self {
    self.parent_mut().schema(schema);
    self
  }

  #[inline]
  fn ident(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.parent_mut().ident(s);
    self
  }

  #[inline]
  fn ident_comma<I, S>(&mut self, s: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    self.parent_mut().ident_comma(s);
    self
  }

  #[inline]
  fn op(&mut self, op: Op) -> &mut Self {
    self.parent_mut().op(op);
    self
  }

  #[inline]
  fn arg(&mut self, arg: impl ToString) -> &mut Self {
    self.parent_mut().arg(arg);
    self
  }

  #[inline]
  fn join(&mut self, builder: impl Builder) -> &mut Self {
    self.parent_mut().join(builder);
    self
  }

  #[inline]
  fn join_many<I, B>(&mut self, builders: I) -> &mut Self
  where
    I: IntoIterator<Item = B>,
    B: Builder,
  {
    self.parent_mut().join_many(builders);
    self
  }

  #[inline]
  fn nested(&mut self, f: impl FnOnce(&mut BaseBuilder)) -> &mut Self {
    self.parent_mut().nested(f);
    self
  }

  #[inline]
  fn push_str(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.parent_mut().push_str(s);
    self
  }

  #[inline]
  fn push(&mut self, c: char) -> &mut Self {
    self.parent_mut().push(c);
    self
  }

  #[inline]
  fn prefix(&mut self, s: impl AsRef<str>) -> &mut Self {
    self.parent_mut().prefix(s);
    self
  }

  #[inline]
  fn comma(&mut self) -> &mut Self {
    self.parent_mut().comma();
    self
  }

  #[inline]
  fn pad(&mut self) -> &mut Self {
    self.parent_mut().pad();
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
  fn set_dialect(&mut self, dialect: Dialect) -> &mut Self {
    self.parent_mut().set_dialect(dialect);
    self
  }

  #[inline]
  fn total(&self) -> usize {
    self.parent().total()
  }

  #[inline]
  fn set_total(&mut self, total: usize) -> &mut Self {
    self.parent_mut().set_total(total);
    self
  }
}

#[inline]
fn is_func(s: &str) -> bool {
  s.contains('(') && s.contains(')')
}

#[inline]
fn is_mod(s: &str) -> bool {
  ["DISTINCT", "ALL", "WITH ROLLUP"]
    .iter()
    .any(|m| s.starts_with(m))
}
