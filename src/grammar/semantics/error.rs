use std::path::PathBuf;

use crate::report::message::{
  line::{HighlightedLine, LineType},
  Message, MessageMeta, MessageType,
};

pub struct ErrorManager<'a> {
  path: PathBuf,
  source: &'a str,
  report: Vec<Message>,
}

impl<'a> ErrorManager<'a> {
  pub fn new(path: &PathBuf, source: &'a str) -> Self {
    Self {
      path: path.clone(),
      source,
      report: vec![],
    }
  }

  pub fn add(&mut self, message: Message) {
    self.report.push(message);
  }

  pub fn make_line(&self, num: usize, typ: LineType) -> HighlightedLine {
    let line = &self.source.lines().nth(num - 1).unwrap_or("");
    HighlightedLine::new(num, line, typ)
  }
  pub fn raw_line(&self, num: usize) -> &str {
    self.source.lines().nth(num - 1).unwrap_or("")
  }

  pub fn make_message(
    &self,
    message: &str,
    typ: MessageType,
    line: usize,
    column: usize,
  ) -> Message {
    Message::new(message, MessageType::Error)
      .with_meta(MessageMeta::FileLocation(self.path.clone(), line, column))
  }
}
