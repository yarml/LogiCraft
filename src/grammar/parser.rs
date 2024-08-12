pub mod ast;
pub mod error;
mod helper;
mod peg;

use super::lexer::token::Token;
use crate::report::location::WithLineInfo;
use ast::Node;
use error::ParserError;
use helper::LineInfoFn;

pub struct Parser;

impl Parser {
  pub fn parse(
    &self,
    tokens: &[WithLineInfo<Token>],
  ) -> Result<Vec<Node>, ParserError> {
    let tokens_ref = tokens.iter().map(|tm| &tm.value).collect::<Vec<_>>();
    let line_info = LineInfoFn::new(tokens);

    peg::parser::global_decl_seq(&tokens_ref, &line_info).map_err(|e| {
      ParserError {
        line: tokens[e.location].line,
        column: tokens[e.location].column,
        len: tokens[e.location].len,
        unexpected: tokens[e.location].value.clone(),
        expected: e.expected,
      }
    })
  }
}
