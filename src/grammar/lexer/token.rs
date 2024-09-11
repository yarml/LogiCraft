use crate::grammar::{
  builtins::Builtin,
  identifier::UnscopedIdentifier,
  keywords::Keyword,
  operators::{AssignOp, Op},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Separator,

  LiteralBoolean(bool),
  LiteralInteger(u64),
  LiteralFloat(f64),
  LiteralCharacter(char),
  LiteralString(String),

  Identifier(UnscopedIdentifier),

  Hash,
  ParenOpen,
  ParenClose,
  BraceOpen,
  BraceClose,
  BracketOpen,
  BracketClose,

  SemiColon,
  Dot,
  Comma,
  Colon,
  Arrow,

  Op(Op),
  AssignOp(AssignOp),

  Keyword(Keyword),
  Builtin(Builtin),
}

impl Token {
  pub fn error_symbol(&self) -> &'static str {
    match self {
      Token::Separator => "space",
      Token::LiteralBoolean(_) => "bool",
      Token::LiteralInteger(_) => "int",
      Token::LiteralFloat(_) => "float",
      Token::LiteralCharacter(_) => "char",
      Token::LiteralString(_) => "string",
      Token::Identifier(_) => "identifier",
      Token::Hash => "#",
      Token::ParenOpen => "(",
      Token::ParenClose => ")",
      Token::BraceOpen => "{",
      Token::BraceClose => "}",
      Token::BracketOpen => "[",
      Token::BracketClose => "]",
      Token::SemiColon => "semicolon",
      Token::Dot => ".",
      Token::Comma => ",",
      Token::Colon => ":",
      Token::Arrow => "->",
      Token::Op(_) => "operator",
      Token::AssignOp(_) => "assignment operator",
      Token::Keyword(kwd) => kwd.error_symbol(),
      Token::Builtin(Builtin::Fn(_)) => "builtin function",
      Token::Builtin(Builtin::Type(_)) => "builtin type",
    }
  }
}
