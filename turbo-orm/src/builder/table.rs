use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, ColumnBuilder, ForeignKeyBuilder};

/// A query builder for `CREATE TABLE` statements.
#[derive(Default)]
pub struct TableBuilder {
  base: BaseBuilder,
  // table name
  name: String,
  // check existence
  exists: bool,
  // table charset
  charset: String,
  // table collation
  collation: String,
  // table options
  options: String,
  // table columns
  columns: Vec<ColumnBuilder>,
  // primary key constraints
  primaries: Vec<String>,
  // foreign keys and indices.
  constraints: Vec<ForeignKeyBuilder>,
}

impl ChildBuilder for TableBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl TableBuilder {
  /// Returns a query builder for the `CREATE TABLE` statement.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Appends the `IF NOT EXISTS` clause to the `CREATE TABLE` statement.
  pub fn if_not_exists(&mut self) -> &mut Self {
    self.exists = true;
    self
  }

  /// Appends the given column to the `CREATE TABLE` statement.
  pub fn column(&mut self, column: ColumnBuilder) -> &mut Self {
    self.columns.push(column);
    self
  }

  /// Adds a column to the primary-key constraint in the statement.
  pub fn primary_key(&mut self, primary: impl Into<String>) -> &mut Self {
    self.primaries.push(primary.into());
    self
  }

  /// Adds a foreign-key to the statement (without constraints).
  pub fn foreign_key(&mut self, key: ForeignKeyBuilder) -> &mut Self {
    self.constraints.push(ForeignKeyBuilder {
      // Erase the constraint symbol/name.
      symbol: String::new(),
      ..key
    });
    self
  }

  /// Adds a foreign-key constraint to the statement.
  pub fn constraint(&mut self, mut constraint: ForeignKeyBuilder) -> &mut Self {
    constraint.prefix("CONSTRAINT ");
    self.constraints.push(constraint);
    self
  }

  /// Sets the `CHARACTER SET` clause to the statement. MySQL only.
  pub fn charset(&mut self, set: impl Into<String>) -> &mut Self {
    self.charset = set.into();
    self
  }

  /// Sets the `COLLATE` clause to the statement. MySQL only.
  pub fn collation(&mut self, collation: impl Into<String>) -> &mut Self {
    self.collation = collation.into();
    self
  }

  /// Adds additional options to to the statement (MySQL only).
  pub fn options(&mut self, options: impl Into<String>) -> &mut Self {
    self.options = options.into();
    self
  }
}

impl Builder for TableBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("CREATE TABLE ");
    if self.exists {
      base.push_str("IF NOT EXISTS ");
    }
    base.ident(self.name);
    let columns = self.columns;
    let primaries = self.primaries;
    let constraints = self.constraints;
    base.nested(|b| {
      b.join_many(columns);
      if !primaries.is_empty() {
        b.comma().push_str("PRIMARY KEY");
        b.nested(|b| {
          b.ident_comma(primaries);
        });
      }
      if !constraints.is_empty() {
        b.comma().join_many(constraints);
      }
    });
    if !self.charset.is_empty() {
      base.push_str(" CHARACTER SET ").push_str(self.charset);
    }
    if !self.collation.is_empty() {
      base.push_str(" COLLATE ").push_str(self.collation);
    }
    if !self.options.is_empty() {
      base.push_str(" ").push_str(self.options);
    }
    (base.buf, base.args)
  }
}
