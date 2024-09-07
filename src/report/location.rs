use super::message::highlight::{Highlight, HighlightType};

#[derive(Debug, Clone)]
pub struct WithLineInfo<T> {
  pub value: T,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl<T> WithLineInfo<T> {
  pub fn make_highlight(
    &self,
    typ: HighlightType,
    label: Option<&str>,
  ) -> Highlight {
    let highlight = Highlight::new(self.column, self.len, typ);
    if let Some(label) = label {
      highlight.with_label(label)
    } else {
      highlight
    }
  }
  pub fn unwrap(self) -> T {
    self.value
  }
  pub fn map<U, F>(self, other: F) -> WithLineInfo<U>
  where
    F: FnOnce(T) -> U,
  {
    WithLineInfo {
      value: other(self.value),
      line: self.line,
      column: self.column,
      len: self.len,
    }
  }
  pub fn try_map<U, E, F>(self, other: F) -> Result<WithLineInfo<U>, E>
  where
    F: FnOnce(T) -> Result<U, E>,
  {
    Ok(WithLineInfo {
      value: other(self.value)?,
      line: self.line,
      column: self.column,
      len: self.len,
    })
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
