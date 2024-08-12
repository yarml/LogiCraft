#[derive(Debug, Clone)]
pub struct WithLineInfo<T> {
  pub value: T,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

#[derive(Debug, Clone)]
pub struct WithRawLineInfo<T> {
  pub value: T,
  pub start: usize,
  pub len: usize,
}

impl<T> PartialEq for WithRawLineInfo<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
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

impl<T> WithRawLineInfo<T> {
  pub fn bake<S>(self, source: S) -> WithLineInfo<T>
  where
    S: FnOnce(usize, usize) -> (usize, usize, usize),
  {
    let (line, column, len) = source(self.start, self.len);
    WithLineInfo {
      value: self.value,
      line,
      column,
      len,
    }
  }
}
