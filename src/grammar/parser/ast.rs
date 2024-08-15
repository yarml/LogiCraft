use crate::{
  grammar::{
    identifier::{CallTarget, Identifier, Name, Type},
    operators::{AssignOp, BinOp, UnOp},
  },
  report::location::WithLineInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedName {
  pub name: WithLineInfo<Name>,
  pub typ: WithLineInfo<Type>,
}

#[derive(Debug, Clone)]
pub enum Expression {
  AtomBoolean(WithLineInfo<bool>),
  AtomInteger(WithLineInfo<isize>),
  AtomFloat(WithLineInfo<f64>),
  AtomString(WithLineInfo<String>),
  AtomIdentifier(WithLineInfo<Identifier>),

  Call(WithLineInfo<CallTarget>, Vec<Expression>),

  UnOp(WithLineInfo<UnOp>, Box<Expression>),
  BinOp(Box<Expression>, WithLineInfo<BinOp>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Node {
  Expression(Expression),
  Assignment(WithLineInfo<Identifier>, WithLineInfo<AssignOp>, Expression),
  VarDecl(WithLineInfo<Name>, Option<WithLineInfo<Type>>, Expression),
  FnDecl(
    WithLineInfo<Name>,
    Vec<TypedName>,
    Option<WithLineInfo<Type>>,
    Vec<Node>,
  ),
  ModDecl(WithLineInfo<Name>),
  StructDecl(WithLineInfo<Name>, Vec<TypedName>),
}
