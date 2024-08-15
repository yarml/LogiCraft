use std::path::PathBuf;

use peg::error::ExpectedSet;

use crate::{
  grammar::lexer::token::Token,
  report::message::{
    highlight::{Highlight, HighlightType},
    line::{HighlightedLine, LineType},
    Message, MessageMeta, MessageType,
  },
};

#[derive(Debug, Clone)]
pub struct ParserError {
  pub line: usize,
  pub column: usize,
  pub len: usize,
  pub unexpected: Token,
  pub expected: ExpectedSet,
}

impl ParserError {
  pub fn get_report(&self, path: &PathBuf, source: &str) -> Message {
    let expected_count = self.expected.tokens().count();
    let expected_list = self.expected.tokens().collect::<Vec<_>>().join(", ");

    let line = source.lines().nth(self.line - 1).unwrap();

    let line =
      HighlightedLine::new(self.line, &line, LineType::Source).with_highlight(
        Highlight::new(self.column, self.len, HighlightType::Focus),
      );
    let expected = if expected_count > 1 {
      format!("Expected one of: {}", expected_list)
    } else {
      format!("Expected: {}", expected_list)
    };

    Message::new(
      &format!("Unexpected token: '{}'", self.unexpected.error_symbol()),
      MessageType::Error,
    )
    .with_meta(MessageMeta::FileLocation(
      path.clone(),
      self.line,
      self.column,
    ))
    .with_line(line)
    .with_note(&expected)
  }
}
