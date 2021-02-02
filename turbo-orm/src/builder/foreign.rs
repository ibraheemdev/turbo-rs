use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, ReferenceBuilder};

#[derive(Default)]
/// The builder for the foreign-key constraint clause.
pub struct ForeignKeyBuilder {
  base: BaseBuilder,
  symbol: String,
  columns: Vec<String>,
  actions: Vec<String>,
  reference: ReferenceBuilder,
}

impl ChildBuilder for ForeignKeyBuilder {
  fn parent(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl ForeignKeyBuilder {
  /// Create a builder for the foreign-key constraint clause in create/alter table statements.
  fn new() -> Self {
    Self::default()
  }

  /// Sets the symbol of the foreign key.
  fn symbol(mut self, symbol: impl Into<String>) -> Self {
    self.symbol = symbol.into();
    self
  }

  /// Sets the columns of the foreign key in the source table.
  fn column(mut self, column: impl Into<String>) -> Self {
    self.columns.push(column.into());
    self
  }

  /// Sets the reference clause.
  fn reference(mut self, reference: ReferenceBuilder) -> Self {
    self.reference = reference;
    self
  }

  /// Sets the on delete action for this constraint.
  fn on_delete(mut self, action: impl AsRef<str>) -> Self {
    let constraint = "ON DELETE ".to_owned();
    constraint.push_str(action.as_ref());
    self.actions.push(constraint);
    self
  }

  /// Sets the on update action for this constraint.
  fn on_update(mut self, action: impl AsRef<str>) -> Self {
    let constraint = "ON UPDATE ".to_owned();
    constraint.push_str(action.as_ref());
    self.actions.push(constraint);
    self
  }
}

impl Builder for ForeignKeyBuilder {
  fn build(self) -> (String, Vec<String>) {
    if !self.symbol.is_empty() {
      self.ident(self.symbol).pad();
    }
    self.push_str("FOREIGN KEY");
    self.nested(|b| {
      b.ident_comma(self.columns);
    });
    self.pad().join(self.reference);
    for action in self.actions {
      self.pad().push_str(action);
    }
    (self.base.buf, self.base.args)
  }
}
