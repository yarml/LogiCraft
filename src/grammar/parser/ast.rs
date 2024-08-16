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
pub struct OptionalTypedName {
  pub name: WithLineInfo<Name>,
  pub typ: Option<WithLineInfo<Type>>,
}

#[derive(Debug, Clone)]
pub enum Expression {
  AtomBoolean(WithLineInfo<bool>),
  AtomInteger(WithLineInfo<isize>),
  AtomFloat(WithLineInfo<f64>),
  AtomString(WithLineInfo<String>),
  AtomIdentifier(Identifier),

  Call(WithLineInfo<CallTarget>, Vec<Expression>),

  UnOp(WithLineInfo<UnOp>, Box<Expression>),
  BinOp(Box<Expression>, WithLineInfo<BinOp>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Node {
  Expression(Expression),
  Assignment {
    target: Identifier,
    op: WithLineInfo<AssignOp>,
    val: Expression,
  },
  VarDecl {
    typ: OptionalTypedName,
    val: Expression,
  },
  FnDecl {
    attributes: Vec<WithLineInfo<Name>>,
    name: WithLineInfo<Name>,
    params: Vec<TypedName>,
    ret_type: Option<WithLineInfo<Type>>,
    body: Vec<Node>,
  },
  ModDecl(WithLineInfo<Name>),
  UseDecl(Identifier),
  StructDecl {
    name: WithLineInfo<Name>,
    fields: Vec<TypedName>,
  },
}
