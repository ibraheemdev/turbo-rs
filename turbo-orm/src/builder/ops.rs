// Represents a predicate operator.
pub enum Op {
  // =
  EQ,
  // <>
  NEQ,
  // >
  GT,
  // >=
  GTE,
  // <
  LT,
  // <=
  LTE,
  // IN
  In,
  // NOT IN
  NotIn,
  // LIKE
  Like,
  // IS NULL
  IsNull,
  // IS NOT NULL
  NotNull,
}

impl Op {
  pub fn as_str(&self) -> &'static str {
    match *self {
      Op::EQ => "=",
      Op::NEQ => "<>",
      Op::GT => ">",
      Op::GTE => ">=",
      Op::LT => "<",
      Op::LTE => "<=",
      Op::In => "IN",
      Op::NotIn => "NOT IN",
      Op::Like => "LIKE",
      Op::IsNull => "IS NULL",
      Op::NotNull => "IS NOT NULL",
    }
  }
}
