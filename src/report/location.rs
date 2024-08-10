use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
  pub file: PathBuf,
  pub line: usize,
  pub column: usize,
}