use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder};

/// A query builder for `DROP INDEX` statements.
#[derive(Default)]
pub struct DropIndexBuilder {
  base: BaseBuilder,
  name: String,
  table: String,
}

impl ChildBuilder for DropIndexBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl DropIndexBuilder {
  /// Creates a builder for the `DROP INDEX` statement.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Defines the table for the index.
  pub fn table(&mut self, table: impl Into<String>) -> &mut Self {
    self.table = table.into();
    self
  }
}

impl Builder for DropIndexBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("DROP INDEX ");
    base.ident(self.name);
    if !self.table.is_empty() {
      base.push_str(" ON ");
      base.ident(self.table);
    }
    (base.buf, base.args)
  }
}
