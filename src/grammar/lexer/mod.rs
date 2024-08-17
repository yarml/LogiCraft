pub mod error;
mod helper;
mod peg;
pub mod token;

use crate::report::line::LineInfoFn;
use crate::report::location::WithLineInfo;
use error::LexerError;
use token::Token;

pub struct Lexer;

impl Lexer {
  pub fn lex(
    &self,
    input: &str,
  ) -> Result<Vec<WithLineInfo<Token>>, LexerError> {
    let line_info = LineInfoFn::new(input);

    peg::lexer::lex(input, &line_info).map_err(|err| LexerError {
      line: err.location.line,
      column: err.location.column - 1,
      len: 2,
    })
  }
}
