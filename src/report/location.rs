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

trait Bake<Q, S> {
  fn bake(self, start: usize, len: usize, source: S) -> WithLineInfo<Q>;
}

impl<T> WithRawLineInfo<T> {
  pub fn bake<Q, S>(self, source: S) -> WithLineInfo<Q>
  where
    T: Bake<Q, S>,
  {
    T::bake(self.value, self.start, self.len, source)
  }
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
