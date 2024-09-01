use crate::{
  grammar::{
    identifier::{CallTarget, Identifier, Name, Type},
    operators::{AssignOp, BinOp, UnOp},
  },
  report::location::WithLineInfo,
};

use super::attributes::Attribute;

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
    attributes: Vec<WithLineInfo<Attribute>>,
    name: WithLineInfo<Name>,
    params: Vec<TypedName>,
    ret_type: Option<WithLineInfo<Type>>,
    body: Vec<Node>,
  },
  Return(Expression),
  ModDecl(WithLineInfo<Name>),
  UseDecl(Identifier),
  StructDecl {
    name: WithLineInfo<Name>,
    fields: Vec<TypedName>,
  },
}

impl Expression {
  pub fn all_identifiers(&self) -> Vec<Identifier> {
    match self {
      Expression::AtomIdentifier(id) => vec![id.clone()],
      Expression::BinOp(left, _, right) => {
        let mut ids = left.all_identifiers();
        ids.extend(right.all_identifiers());
        ids
      }
      Expression::UnOp(_, expr) => expr.all_identifiers(),
      Expression::Call(target, args) => {
        let mut ids = if let CallTarget::Declared(id) = &target.value {
          vec![id.clone()]
        } else {
          vec![]
        };
        ids.extend(args.iter().flat_map(|arg| arg.all_identifiers()));
        ids
      }
      _ => vec![],
    }
  }

  pub fn first_call(&self) -> Option<WithLineInfo<CallTarget>> {
    match self {
      Expression::Call(target, _) => Some(target.clone()),
      Expression::BinOp(left, _, right) => {
        left.first_call().or(right.first_call())
      }
      Expression::UnOp(_, expr) => expr.first_call(),
      _ => None,
    }
  }
}
