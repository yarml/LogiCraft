use std::{collections::HashMap, sync::OnceLock};

peg::parser! {
  pub grammar lexer() for str {
    pub(super) rule comment() = "//" [^ '\n']* "\n"? / "/**/" / "/*" (!"*/" [_])* "*/" / expected!("Comment")
    pub(super) rule whitespace() = quiet! {
      (" " / "\t" / "\n" / "\r" /
      "\u{000B}" / "\u{000C}" / "\u{0085}" / "\u{200E}" /
      "\u{200F}" / "\u{2028}" / "\u{2029}")+
    } / expected!("Whitespace")
    pub(super) rule separator() -> Token = quiet! {
      whitespace() (comment() / whitespace())* { Token::Separator }
    } / expected!("Separator")

    pub(super) rule paren_open() -> Token = quiet! { "(" { Token::ParenOpen } } / expected!("Open Parenthesis")
    pub(super) rule paren_close() -> Token = quiet! { ")" { Token::ParenClose } } / expected!("Close Parenthesis")
    pub(super) rule brace_open() -> Token = quiet! { "{" { Token::BraceOpen } } / expected!("Open Brace")
    pub(super) rule brace_close() -> Token = quiet! { "}" { Token::BraceClose } } / expected!("Close Brace")
    pub(super) rule bracket_open() -> Token = quiet! { "[" { Token::BracketOpen } } / expected!("Open Bracket")
    pub(super) rule bracket_close() -> Token = quiet! { "]" { Token::BracketClose } } / expected!("Close Bracket")

    pub(super) rule brackets() -> Token = quiet! {
      paren_open() / paren_close() / brace_open() / brace_close() / bracket_open() / bracket_close()
    } / expected!("Brackets")

    pub(super) rule semicolon() -> Token = quiet! { ";" { Token::SemiColon } } / expected!("Semi Colon")

    pub(super) rule digit_bin() -> u64 = quiet! { d:$(['0'..='1']) { d.parse().unwrap() } } / expected!("Binary Digit")
    pub(super) rule digit_oct() -> u64 = quiet! { d:$(['0'..='7']) { d.parse().unwrap() } } / expected!("Octal Digit")
    pub(super) rule digit_dec() -> u64 = quiet! { d:$(['0'..='9']) { d.parse().unwrap() } } / expected!("Decimal Digit")
    pub(super) rule digit_hex() -> u64 = quiet! {
      d:$(['0'..='9' | 'a'..='f' | 'A'..='F']) {
        u64::from_str_radix(d, 16).unwrap()
      }
    } / expected!("Hexadecimal Digit")

    pub(super) rule sequence_bin() -> String = quiet! { "0b" n:$(digit_bin() (digit_bin() / "_")*) { n.into() } } / expected!("Binary Sequence")
    pub(super) rule sequence_oct() -> String = quiet! { "0o" n:$(digit_oct() (digit_oct() / "_")*) { n.into() } } / expected!("Octal Sequence")
    pub(super) rule sequence_dec() -> String = quiet! {      n:$(digit_dec() (digit_dec() / "_")*) { n.into() } } / expected!("Decimal Sequence")
    pub(super) rule sequence_hex() -> String = quiet! { "0x" n:$(digit_hex() (digit_hex() / "_")*) { n.into() } } / expected!("Hexadecimal Sequence")

    pub(super) rule literal_bin() -> u64 = quiet! { n:(sequence_bin()) {? parse_literal_integer(&n, 02) } } / expected!("Binary Literal")
    pub(super) rule literal_oct() -> u64 = quiet! { n:(sequence_oct()) {? parse_literal_integer(&n, 08) } } / expected!("Octal Literal")
    pub(super) rule literal_dec() -> u64 = quiet! { n:(sequence_dec()) {? parse_literal_integer(&n, 10) } } / expected!("Decimal Literal")
    pub(super) rule literal_hex() -> u64 = quiet! { n:(sequence_hex()) {? parse_literal_integer(&n, 16) } } / expected!("Hexadecimal Literal")

    pub(super) rule literal_integer() -> Token = quiet! {
      n:(literal_bin() / literal_oct() / literal_hex() / literal_dec()) { Token::LiteralInteger(n) }
    } / expected!("Integer Literal")
    pub(super) rule literal_float() -> Token = quiet! {
      n:$(
        sequence_dec() ("." sequence_dec())? ("e" / "E") ("+" / "-")? sequence_dec() /
        sequence_dec() "." sequence_dec() /
        sequence_dec() "." !"."
      ) {? parse_literal_float(n) }
    } / expected!("Float Literal")

    pub(super) rule char_normal() -> char = quiet! { [^ '\'' | '\\' | '\n' | '\r' | '\t'] } / expected!("Unescaped Character")
    pub(super) rule string_normal() -> char = quiet! { [^ '"' | '\\' | '\n' | '\r' | '\t'] } / expected!("Unescaped Character")

    pub(super) rule escape_quote() -> char = quiet! {
      "\\" c:$("'" / "\"") { c.chars().nth(0).unwrap() }
    } / expected!("Escaped Quote")
    pub(super) rule escape_ascii() -> char = quiet! {
      "\\x" cx:$(digit_oct() digit_hex()) { char::from_u32(u32::from_str_radix(cx, 16).unwrap()).unwrap() }
    } / expected!("Escaped Raw Ascii")
    pub(super) rule escape_unicode() -> char = quiet! {
      "\\u{" n:$((digit_hex() "_"*)*<1, 6>) "}" {? char::from_u32(parse_literal_integer(n, 16).unwrap() as u32).ok_or("") }
    } / expected!("Escaped Raw Unicode")

    pub(super) rule literal_character() -> Token = quiet! {
      "'" c:(char_normal() / escape_quote() / escape_ascii() / escape_unicode()) "'" {  Token::LiteralCharacter(c) }
    } / expected!("Character Literal")
    pub(super) rule literal_string() -> Token = quiet! {
      "\"" s:((string_normal() / escape_ascii() / escape_unicode())*) "\"" { Token::LiteralString(s.iter().collect()) }
    } / expected!("String Literal")

    pub(super) rule identifier_part() -> String = quiet! {
      idp:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) { idp.into() }
    } / expected!("Identifier Part")
    pub(super) rule identifier() -> Token = quiet! {
      root:$("::"?) parts:(identifier_part() ++ "::") {? parse_identifier(root == "::", parts) }
    } / expected!("Identifier")

    pub(super) rule unop_not() -> Token = quiet! {
      "!" { Token::OpNot }
    } / expected!("Operator Not")
    pub(super) rule unop_neg() -> Token = quiet! {
      "-" { Token::OpSub }
    } / expected!("Operator Neg")
    pub(super) rule unop_identity() -> Token = quiet! {
      "+" { Token::OpAdd }
    } / expected!("Operator Identity")
    pub(super) rule unop() -> Token = quiet! {
      unop_not() / unop_neg() / unop_identity()
    } / expected!("Unary Operator")

    pub(super) rule binop_add() -> Token = quiet! {
      "+" { Token::OpAdd }
    } / expected!("Operator Add")
    pub(super) rule binop_sub() -> Token = quiet! {
      "-" { Token::OpSub }
    } / expected!("Operator Sub")
    pub(super) rule binop_mul() -> Token = quiet! {
      "*" { Token::OpMul }
    } / expected!("Operator Mul")
    pub(super) rule binop_div() -> Token = quiet! {
      "/" { Token::OpDiv }
    } / expected!("Operator Div")
    pub(super) rule binop_equals() -> Token = quiet! {
      "==" { Token:: OpEquals }
    } / expected!("Operator Equals")
    pub(super) rule binop_not_equals() -> Token = quiet! {
      "!=" { Token::OpNotEquals }
    } / expected!("Operator Not Equals")
    pub(super) rule binop_greater() -> Token = quiet! {
      ">" { Token::OpGreater }
    } / expected!("Operator Greater")
    pub(super) rule binop_lesser() -> Token = quiet! {
      "<" { Token::OpLesser }
    } / expected!("Operator Lesser")
    pub(super) rule binop_greater_equals() -> Token = quiet! {
      ">=" { Token::OpGreateOrEqual }
    } / expected!("Operator Greater or Equals")
    pub(super) rule binop_lesser_equals() -> Token = quiet! {
      "<=" { Token::OpLesserOrEqual }
    } / expected!("Operator Lesser or Equals")

    pub(super) rule binop() -> Token = quiet! {
      binop_add() /
      binop_sub() /
      binop_mul() /
      binop_div() /
      binop_equals() /
      binop_not_equals() /
      binop_greater_equals() /
      binop_lesser_equals() /
      binop_greater() /
      binop_lesser()
    } / expected!("Binary Operator")

    pub(super) rule assignop_identity() -> Token = quiet! {
      "=" { Token:: OpAssign }
    } / expected!("Operator Assign")
    pub(super) rule assignop_add() -> Token = quiet! {
      "+=" { Token::OpAddAssign }
    } / expected!("Operator Add Assign")
    pub(super) rule assignop_sub() -> Token = quiet! {
      "-=" { Token::OpSubAssign }
    } / expected!("Operator Sub Assign")
    pub(super) rule assignop_mul() -> Token = quiet! {
      "*=" { Token::OpMulAssign }
    } / expected!("Operator Mul Assign")
    pub(super) rule assignop_div() -> Token = quiet! {
      "/=" { Token::OpDivAssign }
    } / expected!("Operator Div Assign")

    pub(super) rule assignop() -> Token = quiet! {
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
      binop() /
      assignop() /
      brackets() /
      semicolon() /
      separator()
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
  parts: Vec<String>,
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
    Ok(Token::Identifier(root, parts))
  }
}

static KEYWORDS_MAP: OnceLock<HashMap<&'static str, Token>> = OnceLock::new();

fn keywords() -> &'static HashMap<&'static str, Token> {
  KEYWORDS_MAP.get_or_init(|| {
    let mut map = HashMap::new();
    map.insert("true", Token::LiteralBoolean(true));
    map.insert("false", Token::LiteralBoolean(false));

    map.insert("let", Token::KeywordLet);
    map.insert("fn", Token::KeywordFn);
    map.insert("if", Token::KeywordIf);
    map.insert("else", Token::KeywordElse);

    map.insert("println", Token::BuiltinPrintLn);

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

  Identifier(bool, Vec<String>),

  ParenOpen,
  ParenClose,
  BraceOpen,
  BraceClose,
  BracketOpen,
  BracketClose,
  SemiColon,

  OpNot,

  OpAdd,
  OpSub,
  OpMul,
  OpDiv,

  OpAssign,

  OpAddAssign,
  OpSubAssign,
  OpMulAssign,
  OpDivAssign,

  OpEquals,
  OpNotEquals,
  OpGreater,
  OpLesser,
  OpGreateOrEqual,
  OpLesserOrEqual,

  KeywordLet,
  KeywordFn,
  KeywordIf,
  KeywordElse,

  BuiltinPrintLn,
}
