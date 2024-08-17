use super::helper::{
  parse_identifier, parse_literal_float, parse_literal_integer,
};
use super::Token;
use crate::grammar::{
  identifier::Name,
  operators::{AssignOp, BinOp, Op, UnOp},
};
use crate::report::line::LineInfoFn;
use crate::report::location::WithLineInfo;

peg::parser! {
  pub(super) grammar lexer(line_info: &LineInfoFn) for str {
    rule comment() = "//" [^ '\n']* "\n"? / "/**/" / "/*" (!"*/" [_])* "*/"
    rule whitespace() =
      (" " / "\t" / "\n" / "\r" /
      "\u{000B}" / "\u{000C}" / "\u{0085}" / "\u{200E}" /
      "\u{200F}" / "\u{2028}" / "\u{2029}")+
    rule separator() -> Token =
      whitespace() (comment() / whitespace())* { Token::Separator }

    rule hash() -> Token = "#" { Token::Hash }
    rule paren_open() -> Token = "(" { Token::ParenOpen }
    rule paren_close() -> Token = ")" { Token::ParenClose }
    rule brace_open() -> Token = "{" { Token::BraceOpen }
    rule brace_close() -> Token = "}" { Token::BraceClose }
    rule bracket_open() -> Token = "[" { Token::BracketOpen }
    rule bracket_close() -> Token = "]" { Token::BracketClose }

    rule brackets() -> Token =
      paren_open() /
      paren_close() /
      brace_open() /
      brace_close() /
      bracket_open() /
      bracket_close()

    rule semicolon() -> Token = ";" { Token::SemiColon }
    rule dot() -> Token = "." { Token::Dot }
    rule comma() -> Token = "," { Token::Comma }
    rule colon() -> Token = ":" { Token::Colon }
    rule arrow() -> Token = "->" { Token::Arrow }

rule digit_bin() -> u64 = d:$(['0'..='1']) { d.parse().unwrap() }    rule digit_oct() -> u64 = d:$(['0'..='7']) { d.parse().unwrap() }
    rule digit_dec() -> u64 = d:$(['0'..='9']) { d.parse().unwrap() }
    rule digit_hex() -> u64 =d:$(['0'..='9' | 'a'..='f' | 'A'..='F']) {
        u64::from_str_radix(d, 16).unwrap()
      }

    rule sequence_bin() -> String =
      "0b" n:$(digit_bin() (digit_bin() / "_")*) { n.into() }
    rule sequence_oct() -> String =
      "0o" n:$(digit_oct() (digit_oct() / "_")*) { n.into() }
    rule sequence_dec() -> String =
           n:$(digit_dec() (digit_dec() / "_")*) { n.into() }
    rule sequence_hex() -> String =
      "0x" n:$(digit_hex() (digit_hex() / "_")*) { n.into() }

    rule literal_bin() -> u64 =
      n:(sequence_bin()) {? parse_literal_integer(&n, 02) }
    rule literal_oct() -> u64 =
      n:(sequence_oct()) {? parse_literal_integer(&n, 08) }
    rule literal_dec() -> u64 =
      n:(sequence_dec()) {? parse_literal_integer(&n, 10) }
    rule literal_hex() -> u64 =
      n:(sequence_hex()) {? parse_literal_integer(&n, 16) }

    rule literal_integer() -> Token =
      n:(
        literal_bin() /
        literal_oct() /
        literal_hex() /
        literal_dec()
      ) { Token::LiteralInteger(n) }
    rule literal_float() -> Token =
      n:$(
        sequence_dec() ("." sequence_dec())?
          ("e" / "E") ("+" / "-")? sequence_dec() /
        sequence_dec() "." sequence_dec() /
        sequence_dec() "." !"."
      ) {? parse_literal_float(n) }

    rule char_normal() -> char = [^ '\'' | '\\' | '\n' | '\r' | '\t']
    rule string_normal() -> char = [^ '"' | '\\' | '\n' | '\r' | '\t']

    rule escape_quote() -> char =
      "\\" c:$("'" / "\"") { c.chars().nth(0).unwrap() }
    rule escape_ascii() -> char =
      "\\x" cx:$(digit_oct() digit_hex()) {
        char::from_u32(u32::from_str_radix(cx, 16).unwrap()).unwrap()
      }
    rule escape_unicode() -> char =
      "\\u{" n:$((digit_hex() "_"*)*<1, 6>) "}" {?
        char::from_u32(parse_literal_integer(n, 16).unwrap() as u32).ok_or("")
      }

    rule literal_character() -> Token =
      "'" c:(
        char_normal() /
        escape_quote() /
        escape_ascii() /
        escape_unicode()
      ) "'" {  Token::LiteralCharacter(c) }
    rule literal_string() -> Token =
      "\"" s:(
        (
          string_normal() /
          escape_ascii() /
          escape_unicode()
        )*
      ) "\"" { Token::LiteralString(s.iter().collect()) }

    rule identifier_part() -> WithLineInfo<Name> =
      start:position!()
      idp:$(
        ['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*
      )
      end:position!() {
        line_info.tag(idp.into(), start, end)
      }
    rule identifier() -> Token =
      root:$("::"?) parts:(identifier_part() ++ "::") {?
        parse_identifier(root == "::", parts)
      }

    rule unop_not() -> Token = "!" { Token::Op(Op::Un(UnOp::Not)) }

    rule op_add() -> Token = "+" { Token::Op(Op::RawAdd) }
    rule op_sub() -> Token = "-" { Token::Op(Op::RawSub) }
    rule binop_mul() -> Token = "*" { Token::Op(Op::Bin(BinOp::Mul)) }
    rule binop_div() -> Token = "/" { Token::Op(Op::Bin(BinOp::Div)) }
    rule binop_mod() -> Token = "%" { Token::Op(Op::Bin(BinOp::Mod)) }
    rule binop_equals() -> Token = "==" { Token::Op(Op::Bin(BinOp::Equal)) }
    rule binop_not_equals() -> Token =
      "!=" { Token::Op(Op::Bin(BinOp::NotEqual)) }
    rule binop_greater() -> Token = ">" { Token::Op(Op::Bin(BinOp::Greater)) }
    rule binop_lesser() -> Token = "<" { Token::Op(Op::Bin(BinOp::Less)) }
    rule binop_greater_equals() -> Token =
      ">=" { Token::Op(Op::Bin(BinOp::GreaterOrEqual)) }
    rule binop_lesser_equals() -> Token =
      "<=" { Token::Op(Op::Bin(BinOp::LessOrEqual)) }


    rule op() -> Token =
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


    rule assignop_identity() -> Token =
      "=" { Token::AssignOp(AssignOp::Identity) }

    rule assignop_add() -> Token =
      "+=" { Token::AssignOp(AssignOp::Add) }

    rule assignop_sub() -> Token =
      "-=" { Token::AssignOp(AssignOp::Sub) }

    rule assignop_mul() -> Token =
      "*=" { Token::AssignOp(AssignOp::Mul) }

    rule assignop_div() -> Token =
      "/=" { Token::AssignOp(AssignOp::Div) }

    rule assignop_mod() -> Token =
      "%=" { Token::AssignOp(AssignOp::Mod) }


    rule assignop() -> Token =
      assignop_identity() /
      assignop_add() /
      assignop_sub() /
      assignop_mul() /
      assignop_div()


    rule any() -> WithLineInfo<Token> =
      start:position!() token:(
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
        colon() /
        hash()
      )
      end:position!() { line_info.tag(token, start, end) }

    pub rule lex() -> Vec<WithLineInfo<Token>> =
        any()*
  }
}
