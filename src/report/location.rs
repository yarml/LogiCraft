use super::message::highlight::{Highlight, HighlightType};

#[derive(Debug, Clone)]
pub struct WithLineInfo<T> {
  pub value: T,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl<T> WithLineInfo<T> {
  pub fn make_highligh(
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
}

impl<T> PartialEq for WithLineInfo<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}
