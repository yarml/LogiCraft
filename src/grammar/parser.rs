use super::{
  builtins::{Builtin, BuiltinFn, BuiltinType},
  identifier::{Identifier, Name, Type, TypedName},
  keywords::Keyword,
  lexer::Token,
  operators::{AssignOp, BinOp, UnOp},
};
use peg::parser;

parser! {
  pub grammar parser<'a>() for [&'a Token] {
    rule name() -> Name = quiet! {
      [Token::Identifier(name)] {?
        if name.is_singular() {
          Ok(name.parts[0].clone())
        } else {
           Err("Expected a singular identifier, found a complex identifier")
          }
        }
    } / expected!("Name");

    rule param_decl() -> TypedName = quiet! {
      name:name() [Token::Separator]? [Token::Colon] [Token::Separator]?
      tn:(
        [Token::Identifier(t)] {
          TypedName(name.clone(), Type::Declared(t.clone()))
        } /
        [Token::Builtin(Builtin::Type(t))] {
          TypedName(name.clone(), Type::Builtin(*t))
        }
      ) { tn }
    } / expected!("Parameter");
    rule param_sep() = quiet! {
      [Token::Separator]? [Token::Comma] [Token::Separator]?
    } / expected!("Parameter Separator");
    rule params_decl() -> Vec<TypedName> = quiet! {
      p:(
        param_decl() ** param_sep()
      ) param_sep()? { p }
    } / expected!("Parameters");

    rule params() -> Vec<ExpressionNode> = quiet! {
      p:(
        expression() ** param_sep()
      ) param_sep()? { p }
    } / expected!("Arguments");

    rule expression() -> ExpressionNode = quiet! {
      (
        [Token::Builtin(Builtin::Fn(b))] [Token::Separator]?
        [Token::ParenOpen] [Token::Separator]?
        args:params() [Token::Separator]?
        [Token::ParenClose] [Token::Separator]?
        { ExpressionNode::BuiltinCall(*b, args) }
      ) /
      [Token::LiteralString(s)] { ExpressionNode::AtomString(s.to_string()) }
    } / expected!("Expression");

    rule statement() -> Node = quiet! {
      e:expression() { Node::Expression(e) } /
      (
        [Token::Identifier(name)] [Token::Separator]?
        [Token::AssignOp(op)] [Token::Separator]?
        e:expression() {
          Node::Assignment(name.clone(), *op, e)
        }
      )
    } / expected!("Statement");

    rule statement_sep() = quiet! {
      [Token::Separator]? [Token::SemiColon] [Token::Separator]?
    } / expected!("Statement Separator");

    rule statements() -> Vec<Node> = quiet! {
      s:(
        statement() ** statement_sep()
      ) statement_sep()? { s }
    } / expected!("Statements");

    pub rule fn_decl() -> Node =
      [Token::Keyword(Keyword::Fn)] [Token::Separator]
      name:name() [Token::Separator]?
      [Token::ParenOpen] [Token::Separator]?
      params:params_decl() [Token::Separator]?
      [Token::ParenClose] [Token::Separator]?
      ret_type:(
        [Token::Arrow] [Token::Separator]? t:(
          [Token::Identifier(t)] { Type::Declared(t.clone()) } /
          [Token::Builtin(Builtin::Type(t))] { Type::Builtin(*t) }
        ) [Token::Separator]? { t }

      )?
      [Token::BraceOpen] [Token::Separator]?
      stmts:statements() [Token::Separator]?
      [Token::BraceClose]
      {
        Node::FnDecl(
          name,
          params,
          ret_type.unwrap_or(
            Type::Builtin(
              BuiltinType::Void
            )
          ),
          stmts
        )
      }
    ;
  }
}

#[derive(Debug, Clone)]
pub enum ExpressionNode {
  AtomInteger(isize),
  AtomString(String),
  AtomBoolean(bool),
  AtomFloat(f64),
  AtomIdentifier(Identifier),

  BuiltinCall(BuiltinFn, Vec<ExpressionNode>),
  FunctionCall(Identifier, Vec<ExpressionNode>),

  UnOp(UnOp, Box<ExpressionNode>),
  BinOp(Box<ExpressionNode>, BinOp, Box<ExpressionNode>),
}

#[derive(Debug, Clone)]
pub enum Node {
  Expression(ExpressionNode),
  Assignment(Identifier, AssignOp, ExpressionNode),
  VarDecl(TypedName, ExpressionNode),
  FnDecl(Name, Vec<TypedName>, Type, Vec<Node>),
  TypeDecl(Name, Vec<TypedName>),
}
