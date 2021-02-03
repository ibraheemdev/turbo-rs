use super::{BaseBuilder, Builder, BuilderExt, ChildBuilder, Dialect};

/// A builder for `INSERT INTO` statement.
#[derive(Default)]
pub struct InsertBuilder {
  base: BaseBuilder,
  table: String,
  schema: String,
  columns: Vec<String>,
  defaults: String,
  returning: Vec<String>,
  values: Vec<Vec<String>>,
}

impl ChildBuilder for InsertBuilder {
  fn parent(&self) -> &BaseBuilder {
    &self.base
  }

  fn parent_mut(&mut self) -> &mut BaseBuilder {
    &mut self.base
  }
}

impl InsertBuilder {
  /// Returns a new `InsertBuilder` for the given table;
  pub fn new(table: impl Into<String>) -> Self {
    Self {
      table: table.into(),
      ..Self::default()
    }
  }

  /// Sets the database name for the insert table.
  pub fn schema(&mut self, schema: impl Into<String>) -> &mut Self {
    self.schema = schema.into();
    self
  }

  /// Syntactic sugar for inserting a single row.
  pub fn set(&mut self, column: impl Into<String>, v: impl ToString) -> &mut Self {
    self.columns.push(column.into());
    match self.values.len() {
      0 => self.values.push(vec![v.to_string()]),
      _ => self.values[0].push(v.to_string()),
    }
    self
  }

  /// Adds a column to the insert statement.
  pub fn value(&mut self, values: Vec<impl ToString>) -> &mut Self {
    self
      .values
      .push(values.iter().map(|v| v.to_string()).collect());
    self
  }
  /// Adds a column to the insert statement.
  pub fn column(&mut self, column: impl Into<String>) -> &mut Self {
    self.columns.push(column.into());
    self
  }

  /// Sets the default values clause based on the dialect type.
  pub fn defaults(&mut self) -> &mut Self {
    self.defaults = match self.dialect() {
      Dialect::MySQL => "VALUES ()",
      _ => "DEFAULT VALUES",
    }
    .to_string();
    self
  }
}

impl Builder for InsertBuilder {
  fn build(self) -> (String, Vec<String>) {
    let mut base = self.base;
    base.push_str("INSERT INTO ");
    base.schema(self.schema);
    base.ident(self.table).pad();
    if !self.defaults.is_empty() && self.columns.is_empty() {
      base.push_str(self.defaults);
    } else {
      let columns = self.columns;
      base.nested(|b| {
        b.ident_comma(columns);
      });
      base.push_str(" VALUES ");
      for (j, _v) in self.values.iter().enumerate() {
        if j > 0 {
          base.comma();
        }
        // TODO: base.nested(|b| b.args(v));
      }
    }
    if !self.returning.is_empty() && base.dialect == Dialect::Postgres {
      base.push_str(" RETURNING ");
      base.ident_comma(self.returning);
    }
    (base.buf, base.args)
  }
}
