use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, ReferenceBuilder};

#[derive(Default)]
/// The builder for the foreign-key constraint clause.
pub struct ForeignKeyBuilder {
  pub(crate) base: BaseBuilder,
  pub(crate) symbol: String,
  pub(crate) columns: Vec<String>,
  pub(crate) actions: Vec<String>,
  pub(crate) reference: ReferenceBuilder,
}

impl ChildBuilder for ForeignKeyBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl ForeignKeyBuilder {
  /// Create a builder for the foreign-key constraint clause in create/alter table statements.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the symbol of the foreign key.
  pub fn symbol(&mut self, symbol: impl Into<String>) -> &mut Self {
    self.symbol = symbol.into();
    self
  }

  /// Sets the columns of the foreign key in the source table.
  pub fn column(&mut self, column: impl Into<String>) -> &mut Self {
    self.columns.push(column.into());
    self
  }

  /// Sets the reference clause.
  pub fn reference(&mut self, reference: ReferenceBuilder) -> &mut Self {
    self.reference = reference;
    self
  }

  /// Sets the on delete action for this constraint.
  pub fn on_delete(&mut self, action: impl AsRef<str>) -> &mut Self {
    let mut constraint = "ON DELETE ".to_owned();
    constraint.push_str(action.as_ref());
    self.actions.push(constraint);
    self
  }

  /// Sets the on update action for this constraint.
  pub fn on_update(&mut self, action: impl AsRef<str>) -> &mut Self {
    let mut constraint = "ON UPDATE ".to_owned();
    constraint.push_str(action.as_ref());
    self.actions.push(constraint);
    self
  }
}

impl Builder for ForeignKeyBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    if !self.symbol.is_empty() {
      base.ident(self.symbol).pad();
    }
    base.push_str("FOREIGN KEY");
    let columns = self.columns;
    base.nested(|b| {
      b.ident_comma(columns);
    });
    base.pad().join(self.reference);
    for action in self.actions {
      base.pad().push_str(action);
    }
    (base.buf, base.args)
  }
}
