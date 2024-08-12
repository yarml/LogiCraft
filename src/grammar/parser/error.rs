use std::path::PathBuf;

use peg::error::ExpectedSet;

use crate::{grammar::lexer::token::Token, report::error::report_and_exit};

#[derive(Debug, Clone)]
pub struct ParserError {
  pub line: usize,
  pub column: usize,
  pub len: usize,
  pub unexpected: Token,
  pub expected: ExpectedSet,
}

impl ParserError {
  pub fn report_and_exit(&self, path: &PathBuf, source: &str) -> ! {
    let line = source.lines().nth(self.line - 1).unwrap();

    let message =
      format!("Unexpecected token: {}", self.unexpected.error_symbol());
    let expected_count = self.expected.tokens().count();
    let expected_list = self.expected.tokens().collect::<Vec<_>>().join(", ");

    let expected = if expected_count > 1 {
      format!("Expected one of: {}", expected_list)
    } else {
      format!("Expected: {}", expected_list)
    };

    report_and_exit(
      line,
      path,
      self.line,
      self.column,
      self.len,
      &message,
      Some(&expected),
      1,
    )
  }
}
