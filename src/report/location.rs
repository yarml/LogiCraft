#[derive(Debug, Clone)]
pub struct WithLineInfo<T> {
  pub value: T,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl<T> PartialEq for WithLineInfo<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}
