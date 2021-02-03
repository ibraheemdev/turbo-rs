use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder};

/// A builder for the reference clause in constraints. For example, in foreign key creation.
#[derive(Default)]
pub struct ReferenceBuilder {
  base: BaseBuilder,
  table: String,
  columns: Vec<String>,
}

impl ChildBuilder for ReferenceBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl ReferenceBuilder {
  /// Create a reference builder for the reference_option clause.
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the referenced table.
  pub fn table(mut self, table: impl Into<String>) -> Self {
    self.table = table.into();
    self
  }

  /// Sets the columns of the referenced table.
  pub fn column(mut self, column: impl Into<String>) -> Self {
    self.columns.push(column.into());
    self
  }
}

impl Builder for ReferenceBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("REFERENCES ");
    base.ident(self.table);
    let columns = self.columns;
    base.nested(|b| {
      b.ident_comma(columns);
    });
    base.build()
  }
}
