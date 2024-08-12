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
