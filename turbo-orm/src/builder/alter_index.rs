use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, Raw};

/// A query builder for `ALTER INDEX` statements.
#[derive(Default)]
pub struct AlterIndexBuilder {
  base: BaseBuilder,
  // index to alter
  name: String,
  // index alteration
  alterations: Vec<String>,
}

impl ChildBuilder for AlterIndexBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl AlterIndexBuilder {
  /// Creates a query builder for a `ALTER INDEX` statement.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Appends the `RENAME TO` clause to the `ALTER INDEX` statement.
  pub fn alter(&mut self, name: impl Into<String>) -> &mut Self {
    let mut query = name.into();
    query.insert_str(0, "RENAME TO ");
    self.alterations.push(query);
    self
  }
}

impl Builder for AlterIndexBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("ALTER INDEX ");
    base.ident(self.name);
    base.pad();
    base.join_many(self.alterations.iter().map(Raw::new));
    base.build()
  }
}
