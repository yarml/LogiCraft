use crate::report::message::{
  highlight::{Highlight, HighlightType},
  line::{HighlightedLine, LineType},
  Message, MessageMeta, MessageType,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LexerError {
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

impl LexerError {
  pub fn get_report(&self, path: &PathBuf, source: &str) -> Message {
    let line = source.lines().nth(self.line - 1).unwrap();

    let line = HighlightedLine::new(self.line, &line, LineType::Source)
      .with_highlight(
        Highlight::new(self.column, self.len, HighlightType::Focus)
          .with_label("here"),
      );

    Message::new("Unexpected token", MessageType::Error)
      .with_meta(MessageMeta::FileLocation(
        path.clone(),
        self.line,
        self.column,
      ))
      .with_line(line)
  }
}
