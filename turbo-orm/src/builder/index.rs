use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder};

/// A query builder for `CREATE INDEX` statements.
#[derive(Default)]
pub struct IndexBuilder {
  base: BaseBuilder,
  name: String,
  table: String,
  unique: bool,
  columns: Vec<String>,
}

impl ChildBuilder for IndexBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl IndexBuilder {
  /// Creates a builder for the `CREATE INDEX` statement.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Sets the index to be a unique index.
  pub fn unique(&mut self) -> &mut Self {
    self.unique = true;
    self
  }

  /// Defines the table for the index.
  pub fn table(&mut self, table: impl Into<String>) -> &mut Self {
    self.table = table.into();
    self
  }

  /// Appends the given columns to the column list for the index.
  pub fn column(&mut self, column: impl Into<String>) -> &mut Self {
    self.columns.push(column.into());
    self
  }
}

impl Builder for IndexBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("CREATE ");
    if self.unique {
      base.push_str("UNIQUE ");
    }
    base.push_str("INDEX ");
    base.ident(self.name);
    base.push_str(" ON ");
    let columns = self.columns;
    base.ident(self.table).nested(|b| {
      b.ident_comma(columns);
    });
    (base.buf, base.args)
  }
}
