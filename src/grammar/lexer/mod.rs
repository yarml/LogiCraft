pub mod error;
mod helper;
mod peg;
pub mod token;

use crate::report::location::WithLineInfo;
use error::LexerError;
use token::Token;

pub struct Lexer;

impl Lexer {
  pub fn lex(
    &self,
    input: &str,
  ) -> Result<Vec<WithLineInfo<Token>>, LexerError> {
    let newlines = input
      .char_indices()
      .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
      .collect::<Vec<_>>();

    let line_info = |start, end| {
      let len = end - start;
      let line = newlines
        .iter()
        .position(|&i| i > start)
        .unwrap_or(newlines.len());
      let column = newlines
        .get(line.wrapping_sub(1))
        .map(|&i| start - i)
        .unwrap_or(start);
      (line + 1, column, len)
    };

    peg::lexer::lex(input, &line_info).map_err(|err| LexerError {
      line: err.location.line,
      column: err.location.column - 1,
      len: 2,
    })
  }
}
