use std::{fmt::Display, hash::Hash};

#[derive(Debug, Clone)]
pub struct WithLineInfo<T> {
  pub value: T,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl<T> WithLineInfo<T> {
  pub fn map<F, U>(self, mapfn: F) -> WithLineInfo<U>
  where
    F: FnOnce(T) -> U,
  {
    WithLineInfo {
      value: mapfn(self.value),
      line: self.line,
      column: self.column,
      len: self.len,
    }
  }
}

impl<T> PartialEq for WithLineInfo<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}
impl<T> Eq for WithLineInfo<T> where T: Eq {}

impl<T> Hash for WithLineInfo<T>
where
  T: Hash,
{
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.value.hash(state);
  }
}

impl<T> Display for WithLineInfo<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.value.fmt(f)
  }
}
