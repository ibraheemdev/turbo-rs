use super::foreign::ForeignKeyBuilder;
use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, Dialect};

/// The builder for column definitions in table creation
#[derive(Default)]
pub struct ColumnBuilder {
  base: BaseBuilder,
  // column type
  typ: String,
  // column name
  name: String,
  // extra attributes
  attr: String,
  // modify existing
  modify: bool,
  // foreign-key constraint
  foreign: Option<ForeignKeyBuilder>,
  // column checks
  check: Option<Box<dyn Fn(&mut BaseBuilder)>>,
}

impl ChildBuilder for ColumnBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl ColumnBuilder {
  /// Returns a new `ColumnBuilder` with the given name.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      check: None,
      ..Self::default()
    }
  }

  /// Sets the column type.
  pub fn typ(&mut self, typ: impl Into<String>) -> &mut Self {
    self.typ = typ.into();
    self
  }

  /// Sets an extra attribute for the column, like `UNIQUE` or `AUTO_INCREMENT`.
  pub fn attr(&mut self, attr: &str) -> &mut Self {
    if !self.attr.is_empty() && !attr.is_empty() {
      self.attr.push(' ');
    }
    self.attr.push_str(attr);
    self
  }

  /// Adds the `CONSTRAINT` clause to the `ADD COLUMN` statement in SQLite.
  pub fn constraint(&mut self, foreign: impl Into<Option<ForeignKeyBuilder>>) -> &mut Self {
    self.foreign = foreign.into();
    self
  }

  /// Adds a CHECK clause to the ADD COLUMN statement.
  pub fn check<O, C>(&mut self, check: O) -> &mut Self
  where
    O: Into<Option<C>>,
    C: Fn(&mut BaseBuilder) + 'static,
  {
    self.check = match check.into() {
      Some(c) => Some(Box::new(c)),
      None => None,
    };
    self
  }
}

impl Builder for ColumnBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.ident(self.name);
    if !self.typ.is_empty() {
      if base.dialect == Dialect::Postgres && self.modify {
        base.push_str(" TYPE");
      }
      base.pad().push_str(self.typ);
    }
    if !self.attr.is_empty() {
      base.pad().push_str(self.attr);
    }
    if let Some(foreign) = self.foreign {
      base.push_str(" CONSTRAINT ").push_str(foreign.symbol);
      base.pad().join(foreign.reference);
      for action in foreign.actions {
        base.pad().push_str(action);
      }
    }
    if let Some(check) = self.check {
      base.push_str(" CHECK ");
      base.nested(check);
    }
    (base.buf, base.args)
  }
}
