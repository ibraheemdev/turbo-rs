use super::{
  BaseBuilder, Builder, BuilderExt, ChildBuilder, ColumnBuilder, Dialect, ForeignKeyBuilder, Raw,
};

/// A query builder for `ALTER TABLE` statement.
#[derive(Default)]
pub struct TableAlterBuilder {
  base: BaseBuilder,
  // table to alter
  name: String,
  // table alteration
  alterations: Vec<String>,
}

impl ChildBuilder for TableAlterBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl TableAlterBuilder {
  /// Creates a query builder for the `ALTER TABLE` statement.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Appends the `ADD COLUMN` clause to the given `ALTER TABLE` statement.
  pub fn add_column(&mut self, mut column: ColumnBuilder) -> &mut Self {
    column.prefix("ADD COLUMN ");
    let (column, _) = column.build();
    self.alterations.push(column);
    self
  }

  /// Appends the `MODIFY/ALTER COLUMN` clause to the given `ALTER TABLE` statement.
  pub fn modify_column(&mut self, mut column: ColumnBuilder) -> &mut Self {
    column.prefix(match self.dialect() {
      Dialect::Postgres => "ALTER COLUMN ",
      _ => "MODIFY COLUMN ",
    });
    let (column, _) = column.build();
    self.alterations.push(column);
    self
  }

  /// Appends the `RENAME COLUMN` clause to the given `ALTER TABLE` statement.
  pub fn rename_column(&mut self, old: impl Into<String>, new: impl Into<String>) -> &mut Self {
    self.alterations.push(format!(
      "RENAME COLUMN {} to {}",
      self.quote(old),
      self.quote(new)
    ));
    self
  }

  /// Appends the `DROP COLUMN` clause to the given `ALTER TABLE` statement.
  pub fn drop_column(&mut self, mut column: ColumnBuilder) -> &mut Self {
    column.prefix("DROP COLUMN ");
    let (column, _) = column.build();
    self.alterations.push(column);
    self
  }

  /// Appends the `RENAME INDEX` clause to the given `ALTER TABLE` statement.
  pub fn rename_index(&mut self, old: impl Into<String>, new: impl Into<String>) -> &mut Self {
    self.alterations.push(format!(
      "RENAME INDEX {} to {}",
      self.quote(old),
      self.quote(new)
    ));
    self
  }

  /// Appends the `DROP INDEX` clause to the given `ALTER TABLE` statement.
  pub fn drop_index(&mut self, name: impl AsRef<str>) -> &mut Self {
    let mut query = "DROP INDEX ".to_owned();
    query.push_str(&self.quote(name.as_ref()));
    self.alterations.push(query);
    self
  }

  // TODO: Appends the `ADD INDEX` clause to the given `ALTER TABLE` statement.
  // pub fn add_index(&mut self, index: IndexBuilder) -> &mut Self {}

  /// Adds a foreign key constraint to the given `ALTER TABLE` statement.
  pub fn add_foreign_key(&mut self, mut fk: ForeignKeyBuilder) -> &mut Self {
    fk.prefix("ADD CONSTRAINT ");
    let (fk, _) = fk.build();
    self.alterations.push(fk);
    self
  }
  /// Appends the `DROP CONSTRAINT` clause to the given `ALTER TABLE` statement.
  pub fn drop_constraint(&mut self, ident: impl AsRef<str>) -> &mut Self {
    let mut query = "DROP CONSTRAINT ".to_owned();
    query.push_str(&self.quote(ident.as_ref()));
    self.alterations.push(query);
    self
  }

  /// Appends the `DROP FOREIGN KEY` clause to the given `ALTER TABLE` statement.
  pub fn drop_foreign_key(&mut self, ident: impl AsRef<str>) -> &mut Self {
    let mut query = "DROP FOREIGN KEY ".to_owned();
    query.push_str(&self.quote(ident.as_ref()));
    self.alterations.push(query);
    self
  }
}

impl Builder for TableAlterBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str(self.name);
    base.pad();
    base.join_many(self.alterations.iter().map(Raw::new));
    (base.buf, base.args)
  }
}
