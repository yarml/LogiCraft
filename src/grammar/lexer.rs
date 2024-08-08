use std::{collections::HashMap, sync::OnceLock};

use super::{
  builtins::{Builtin, BuiltinFn, BuiltinType},
  identifier::{Identifier, Name},
  keywords::Keyword,
  operators::{AssignOp, BinOp, Op, UnOp},
};

peg::parser! {
  pub grammar lexer() for str {
    rule comment() = quiet! {
      "//" [^ '\n']* "\n"? / "/**/" / "/*" (!"*/" [_])* "*/"
    } / expected!("Comment")
    rule whitespace() = quiet! {
      (" " / "\t" / "\n" / "\r" /
      "\u{000B}" / "\u{000C}" / "\u{0085}" / "\u{200E}" /
      "\u{200F}" / "\u{2028}" / "\u{2029}")+
    } / expected!("Whitespace")
    rule separator() -> Token = quiet! {
      whitespace() (comment() / whitespace())* { Token::Separator }
    } / expected!("Separator")

    rule paren_open() -> Token = quiet! {
      "(" { Token::ParenOpen }
    } / expected!("Open Parenthesis")
    rule paren_close() -> Token = quiet! {
      ")" { Token::ParenClose }
    } / expected!("Close Parenthesis")
    rule brace_open() -> Token = quiet! {
      "{" { Token::BraceOpen }
    } / expected!("Open Brace")
    rule brace_close() -> Token = quiet! {
      "}" { Token::BraceClose }
    } / expected!("Close Brace")
    rule bracket_open() -> Token = quiet! {
      "[" { Token::BracketOpen }
    } / expected!("Open Bracket")
    rule bracket_close() -> Token = quiet! {
      "]" { Token::BracketClose }
    } / expected!("Close Bracket")

    rule brackets() -> Token = quiet! {
      paren_open() /
      paren_close() /
      brace_open() /
      brace_close() /
      bracket_open() /
      bracket_close()
    } / expected!("Brackets")

    rule semicolon() -> Token = quiet! {
      ";" { Token::SemiColon }
    } / expected!("Semi Colon")
    rule dot() -> Token = quiet! {
      "." { Token::Dot }
    } / expected!("Dot")
    rule comma() -> Token = quiet! {
      "," { Token::Comma }
    } / expected!("Comma")
    rule colon() -> Token = quiet! {
      ":" { Token::Colon }
    } / expected!("Colon")
    rule arrow() -> Token = quiet! {
      "->" { Token::Arrow }
    } / expected!("Arrow")

    rule digit_bin() -> u64 = quiet! {
      d:$(['0'..='1']) { d.parse().unwrap() }
    } / expected!("Binary Digit")
    rule digit_oct() -> u64 = quiet! {
      d:$(['0'..='7']) { d.parse().unwrap() }
    } / expected!("Octal Digit")
    rule digit_dec() -> u64 = quiet! {
      d:$(['0'..='9']) { d.parse().unwrap() }
    } / expected!("Decimal Digit")
    rule digit_hex() -> u64 = quiet! {
      d:$(['0'..='9' | 'a'..='f' | 'A'..='F']) {
        u64::from_str_radix(d, 16).unwrap()
      }
    } / expected!("Hexadecimal Digit")

    rule sequence_bin() -> String = quiet! {
      "0b" n:$(digit_bin() (digit_bin() / "_")*) { n.into() }
    } / expected!("Binary Sequence")
    rule sequence_oct() -> String = quiet! {
      "0o" n:$(digit_oct() (digit_oct() / "_")*) { n.into() }
    } / expected!("Octal Sequence")
    rule sequence_dec() -> String = quiet! {
           n:$(digit_dec() (digit_dec() / "_")*) { n.into() }
    } / expected!("Decimal Sequence")
    rule sequence_hex() -> String = quiet! {
      "0x" n:$(digit_hex() (digit_hex() / "_")*) { n.into() }
    } / expected!("Hexadecimal Sequence")

    rule literal_bin() -> u64 = quiet! {
      n:(sequence_bin()) {? parse_literal_integer(&n, 02) }
    } / expected!("Binary Literal")
    rule literal_oct() -> u64 = quiet! {
      n:(sequence_oct()) {? parse_literal_integer(&n, 08) }
    } / expected!("Octal Literal")
    rule literal_dec() -> u64 = quiet! {
      n:(sequence_dec()) {? parse_literal_integer(&n, 10) }
    } / expected!("Decimal Literal")
    rule literal_hex() -> u64 = quiet! {
      n:(sequence_hex()) {? parse_literal_integer(&n, 16) }
    } / expected!("Hexadecimal Literal")

    rule literal_integer() -> Token = quiet! {
      n:(
        literal_bin() /
        literal_oct() /
        literal_hex() /
        literal_dec()
      ) { Token::LiteralInteger(n) }
    } / expected!("Integer Literal")
    rule literal_float() -> Token = quiet! {
      n:$(
        sequence_dec() ("." sequence_dec())?
          ("e" / "E") ("+" / "-")? sequence_dec() /
        sequence_dec() "." sequence_dec() /
        sequence_dec() "." !"."
      ) {? parse_literal_float(n) }
    } / expected!("Float Literal")

    rule char_normal() -> char = quiet! {
      [^ '\'' | '\\' | '\n' | '\r' | '\t']
    } / expected!("Unescaped Character")
    rule string_normal() -> char = quiet! {
      [^ '"' | '\\' | '\n' | '\r' | '\t']
    } / expected!("Unescaped Character")

    rule escape_quote() -> char = quiet! {
      "\\" c:$("'" / "\"") { c.chars().nth(0).unwrap() }
    } / expected!("Escaped Quote")
    rule escape_ascii() -> char = quiet! {
      "\\x" cx:$(digit_oct() digit_hex()) {
        char::from_u32(u32::from_str_radix(cx, 16).unwrap()).unwrap()
      }
    } / expected!("Escaped Raw Ascii")
    rule escape_unicode() -> char = quiet! {
      "\\u{" n:$((digit_hex() "_"*)*<1, 6>) "}" {?
        char::from_u32(parse_literal_integer(n, 16).unwrap() as u32).ok_or("")
      }
    } / expected!("Escaped Raw Unicode")

    rule literal_character() -> Token = quiet! {
      "'" c:(
        char_normal() /
        escape_quote() /
        escape_ascii() /
        escape_unicode()
      ) "'" {  Token::LiteralCharacter(c) }
    } / expected!("Character Literal")
    rule literal_string() -> Token = quiet! {
      "\"" s:(
        (
          string_normal() /
          escape_ascii() /
          escape_unicode()
        )*
      ) "\"" { Token::LiteralString(s.iter().collect()) }
    } / expected!("String Literal")

    rule identifier_part() -> String = quiet! {
      idp:$(
        ['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*
      ) { idp.into() }
    } / expected!("Identifier Part")
    rule identifier() -> Token = quiet! {
      root:$("::"?) parts:(identifier_part() ++ "::") {?
        parse_identifier(root == "::", parts)
      }
    } / expected!("Identifier")

    rule unop_not() -> Token = quiet! {
      "!" { Token::Op(Op::Un(UnOp::Not)) }
    } / expected!("Operator Not")

    rule op_add() -> Token = quiet! {
      "+" { Token::Op(Op::RawAdd) }
    } / expected!("Operator Add")
    rule op_sub() -> Token = quiet! {
      "-" { Token::Op(Op::RawSub) }
    } / expected!("Operator Sub")
    rule binop_mul() -> Token = quiet! {
      "*" { Token::Op(Op::Bin(BinOp::Mul)) }
    } / expected!("Operator Mul")
    rule binop_div() -> Token = quiet! {
      "/" { Token::Op(Op::Bin(BinOp::Div)) }
    } / expected!("Operator Div")
    rule binop_mod() -> Token = quiet! {
      "%" { Token::Op(Op::Bin(BinOp::Mod)) }
    } / expected!("Operator Mod")
    rule binop_equals() -> Token = quiet! {
      "==" { Token::Op(Op::Bin(BinOp::Equal)) }
    } / expected!("Operator Equals")
    rule binop_not_equals() -> Token = quiet! {
      "!=" { Token::Op(Op::Bin(BinOp::NotEqual)) }
    } / expected!("Operator Not Equals")
    rule binop_greater() -> Token = quiet! {
      ">" { Token::Op(Op::Bin(BinOp::Greater)) }
    } / expected!("Operator Greater")
    rule binop_lesser() -> Token = quiet! {
      "<" { Token::Op(Op::Bin(BinOp::Less)) }
    } / expected!("Operator Lesser")
    rule binop_greater_equals() -> Token = quiet! {
      ">=" { Token::Op(Op::Bin(BinOp::GreaterOrEqual)) }
    } / expected!("Operator Greater or Equals")
    rule binop_lesser_equals() -> Token = quiet! {
      "<=" { Token::Op(Op::Bin(BinOp::LessOrEqual)) }
    } / expected!("Operator Lesser or Equals")

    rule op() -> Token = quiet! {
      unop_not() /
      op_add() /
      op_sub() /
      binop_mul() /
      binop_div() /
      binop_equals() /
      binop_not_equals() /
      binop_greater_equals() /
      binop_lesser_equals() /
      binop_greater() /
      binop_lesser()
    } / expected!("Binary Operator")

    rule assignop_identity() -> Token = quiet! {
      "=" { Token::AssignOp(AssignOp::Identity) }
    } / expected!("Operator Assign")
    rule assignop_add() -> Token = quiet! {
      "+=" { Token::AssignOp(AssignOp::Add) }
    } / expected!("Operator Add Assign")
    rule assignop_sub() -> Token = quiet! {
      "-=" { Token::AssignOp(AssignOp::Sub) }
    } / expected!("Operator Sub Assign")
    rule assignop_mul() -> Token = quiet! {
      "*=" { Token::AssignOp(AssignOp::Mul) }
    } / expected!("Operator Mul Assign")
    rule assignop_div() -> Token = quiet! {
      "/=" { Token::AssignOp(AssignOp::Div) }
    } / expected!("Operator Div Assign")
    rule assignop_mod() -> Token = quiet! {
      "%=" { Token::AssignOp(AssignOp::Mod) }
    } / expected!("Operator Mod Assign")

    rule assignop() -> Token = quiet! {
      assignop_identity() /
      assignop_add() /
      assignop_sub() /
      assignop_mul() /
      assignop_div()
    } / expected!("Assignment Operator")

    rule any() -> Token = quiet! {
      literal_integer() /
      literal_float() /
      literal_character() /
      literal_string() /
      identifier() /
      arrow() /
      assignop() /
      op() /
      brackets() /
      semicolon() /
      separator() /
      dot() /
      comma() /
      colon()
    } / expected!("Any Token")
    pub rule lex() -> Vec<Token> = any()*
  }
}

fn parse_literal_integer(
  literal: &str,
  base: u32,
) -> Result<u64, &'static str> {
  let literal = literal.replace("_", "");
  Ok(u64::from_str_radix(&literal, base).or(Err(""))?)
}
fn parse_literal_float(literal: &str) -> Result<Token, &'static str> {
  let literal = literal.replace("_", "");
  Ok(Token::LiteralFloat(literal.parse().or(Err(""))?))
}
fn parse_identifier(
  root: bool,
  parts: Vec<Name>,
) -> Result<Token, &'static str> {
  let kw = parts.iter().find_map(|part| {
    let kws = keywords();
    if let Some(token) = kws.get(part.as_str()) {
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

fn keywords() -> &'static HashMap<&'static str, Token> {
  KEYWORDS_MAP.get_or_init(|| {
    let mut map = HashMap::new();
    map.insert("true", Token::LiteralBoolean(true));
    map.insert("false", Token::LiteralBoolean(false));

    map.insert("mod", Token::Keyword(Keyword::Mod));
    map.insert("fn", Token::Keyword(Keyword::Fn));
    map.insert("let", Token::Keyword(Keyword::Let));
    map.insert("if", Token::Keyword(Keyword::If));
    map.insert("else", Token::Keyword(Keyword::Else));
    map.insert("return", Token::Keyword(Keyword::Ret));

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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Separator,

  LiteralBoolean(bool),
  LiteralInteger(u64),
  LiteralFloat(f64),
  LiteralCharacter(char),
  LiteralString(String),

  Identifier(Identifier),

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
