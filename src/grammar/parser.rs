use super::{
  builtins::Builtin,
  identifier::{Identifier, Name, Type, TypedName},
  lexer::Token,
  operators::{AssignOp, BinOp, UnOp},
};
use peg::parser;

parser! {
  pub grammar parser() for [Token] {

  }
}

pub enum ExpressionNode {
  AtomInteger(isize),
  AtomString(String),
  AtomBoolean(bool),
  AtomFloat(f64),
  AtomIdentifier(Identifier),

  BuiltinCall(Builtin, Vec<ExpressionNode>),
  FunctionCall(Identifier, Vec<ExpressionNode>),

  UnOp(UnOp, Box<ExpressionNode>),
  BinOp(Box<ExpressionNode>, BinOp, Box<ExpressionNode>),
}

pub enum Node {
  Expression(ExpressionNode),
  Assignment(Identifier, AssignOp, ExpressionNode),
  VarDecl(TypedName, ExpressionNode),
  FnDecl(Name, Vec<TypedName>, Type, Vec<Node>),
  TypeDecl(Name, Vec<TypedName>),
}
