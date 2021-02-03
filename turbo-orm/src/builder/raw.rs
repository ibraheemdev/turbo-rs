use super::Builder;

// A raw SQL statement that is placed as-is in the query.
pub struct Raw<T: Into<String>>(T);

impl<T: Into<String>> Raw<T> {
  pub fn new(inner: T) -> Self {
    Self(inner)
  }
}

impl<T: Into<String>> Builder for Raw<T> {
  #[inline]
  fn build(self) -> (String, Vec<String>) {
    (self.0.into(), Vec::new())
  }
}
