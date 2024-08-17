use super::location::WithLineInfo;

pub struct LineInfoFn {
  newlines: Vec<usize>,
}

impl LineInfoFn {
  pub fn new(source: &str) -> Self {
    let newlines = source
      .char_indices()
      .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
      .collect::<Vec<_>>();
    Self { newlines }
  }
  pub fn get_line_info(
    &self,
    start: usize,
    end: usize,
  ) -> (usize, usize, usize) {
    let len = end - start;
    let line = self
      .newlines
      .iter()
      .position(|&i| i > start)
      .unwrap_or(self.newlines.len())
      + 1;
    let column = self
      .newlines
      .get(line.wrapping_sub(2))
      .map(|&i| start - i)
      .unwrap_or(start);
    (line, column, len)
  }
  pub fn tag<T>(&self, value: T, start: usize, end: usize) -> WithLineInfo<T> {
    let (line, column, len) = self.get_line_info(start, end);
    WithLineInfo {
      value,
      line,
      column,
      len,
    }
  }
}
