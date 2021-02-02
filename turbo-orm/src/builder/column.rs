use super::foreign::ForeignKeyBuilder;
use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, Dialect};

/// The builder for column definitions in table creation
pub struct ColumnBuilder<F: Fn(&mut BaseBuilder)> {
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
  check: Option<F>,
}

impl<F: Fn(&mut BaseBuilder)> ChildBuilder for ColumnBuilder<F> {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl<F: Fn(&mut BaseBuilder)> Default for ColumnBuilder<F> {
  fn default() -> Self {
    Self {
      base: BaseBuilder::default(),
      typ: String::new(),
      name: String::new(),
      attr: String::new(),
      modify: false,
      foreign: None,
      check: None,
    }
  }
}

impl<F: Fn(&mut BaseBuilder)> ColumnBuilder<F> {
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
      self.attr.push_str(" ");
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
  pub fn check<FF: Fn(&mut BaseBuilder)>(self, check: FF) -> ColumnBuilder<FF> {
    ColumnBuilder {
      typ: self.typ,
      name: self.name,
      base: self.base,
      modify: self.modify,
      foreign: self.foreign,
      attr: self.attr,
      check: Some(check),
    }
  }
}

impl<F: Fn(&mut BaseBuilder)> Builder for ColumnBuilder<F> {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.ident(self.name);
    if !self.typ.is_empty() {
      if base.dialect == Dialect::Postgres {
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
