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

    fn line_column(newlines: &[usize], start: usize) -> (usize, usize) {
      let line = newlines
        .iter()
        .position(|&i| i > start)
        .unwrap_or(newlines.len());
      let column = newlines
        .get(line.wrapping_sub(1))
        .map(|&i| start - i)
        .unwrap_or(start);
      (line + 1, column)
    }

    match peg::lexer::lex(input) {
      Err(err) => {
        let (line, column) = line_column(&newlines, err.location.offset);
        Err(LexerError {
          line,
          column,
          len: err.location.offset,
        })
      }
      Ok(tokens) => Ok(
        tokens
          .into_iter()
          .map(|rtm| {
            let (line, column) = line_column(&newlines, rtm.start);
            WithLineInfo {
              value: rtm.value,
              line,
              column,
              len: rtm.len,
            }
          })
          .collect(),
      ),
    }
  }
}
