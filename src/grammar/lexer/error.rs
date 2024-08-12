use crate::report::error::report_and_exit;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LexerError {
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl LexerError {
  pub fn report_and_exit(&self, path: &PathBuf, source: &str) -> ! {
    let line = source.lines().nth(self.line - 1).unwrap();
    report_and_exit(
      line,
      &path,
      self.line,
      self.column,
      self.len,
      "Unexpected token",
      None,
      1,
    );
  }
}
