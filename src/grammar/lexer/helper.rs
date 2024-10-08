use super::token::Token;
use crate::{
  grammar::{
    builtins::{Builtin, BuiltinFn, BuiltinType},
    identifier::{Identifier, Name},
    keywords::Keyword,
  },
  report::location::WithLineInfo,
};
use std::{collections::HashMap, sync::OnceLock};

pub fn parse_literal_integer(
  literal: &str,
  base: u32,
) -> Result<u64, &'static str> {
  let literal = literal.replace("_", "");
  Ok(u64::from_str_radix(&literal, base).or(Err(""))?)
}
pub fn parse_literal_float(literal: &str) -> Result<Token, &'static str> {
  let literal = literal.replace("_", "");
  Ok(Token::LiteralFloat(literal.parse().or(Err(""))?))
}
pub fn parse_identifier(
  root: bool,
  parts: Vec<WithLineInfo<Name>>,
) -> Result<Token, &'static str> {
  let kw = parts.iter().find_map(|part| {
    let kws = keywords();
    if let Some(token) = kws.get(part.value.as_str()) {
      Some(token.clone())
    } else {
      None
    }
  });
  if let Some(token) = kw {
    if root || parts.len() != 1 {
      Err("")
    } else {
      Ok(token)
    }
  } else {
    Ok(Token::Identifier(Identifier { root, parts }))
  }
}

static KEYWORDS_MAP: OnceLock<HashMap<&'static str, Token>> = OnceLock::new();

pub fn keywords() -> &'static HashMap<&'static str, Token> {
  KEYWORDS_MAP.get_or_init(|| {
    let mut map = HashMap::new();
    map.insert("true", Token::LiteralBoolean(true));
    map.insert("false", Token::LiteralBoolean(false));

    map.insert("mod", Token::Keyword(Keyword::Mod));
    map.insert("use", Token::Keyword(Keyword::Use));
    map.insert("fn", Token::Keyword(Keyword::Fn));
    map.insert("let", Token::Keyword(Keyword::Let));
    map.insert("mut", Token::Keyword(Keyword::Mut));
    map.insert("if", Token::Keyword(Keyword::If));
    map.insert("else", Token::Keyword(Keyword::Else));
    map.insert("return", Token::Keyword(Keyword::Ret));

    map.insert("struct", Token::Keyword(Keyword::Struct));

    map.insert("void", Token::Builtin(Builtin::Type(BuiltinType::Void)));
    map.insert("bool", Token::Builtin(Builtin::Type(BuiltinType::Bool)));
    map.insert("int", Token::Builtin(Builtin::Type(BuiltinType::Int)));
    map.insert("float", Token::Builtin(Builtin::Type(BuiltinType::Float)));
    map.insert("char", Token::Builtin(Builtin::Type(BuiltinType::Char)));
    map.insert("string", Token::Builtin(Builtin::Type(BuiltinType::String)));

    map.insert("println", Token::Builtin(Builtin::Fn(BuiltinFn::PrintLn)));

    map
  })
}
