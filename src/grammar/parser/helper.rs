use crate::{grammar::lexer::token::Token, report::location::WithLineInfo};

pub struct LineInfoFn<'a> {
  tokens: &'a [WithLineInfo<Token>],
}

impl<'a> LineInfoFn<'a> {
  pub fn new(tokens: &'a [WithLineInfo<Token>]) -> Self {
    Self { tokens }
  }

  pub fn tag<T>(&self, value: T, start: usize, end: usize) -> WithLineInfo<T> {
    let len = end - start;
    let line = self.tokens[start].line;
    let column = self.tokens[start].column;
    let len: usize = self.tokens[start..start + len]
      .iter()
      .map(|token| token.len)
      .sum();
    WithLineInfo {
      value,
      line,
      column,
      len,
    }
  }
}
