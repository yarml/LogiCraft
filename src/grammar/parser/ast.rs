use crate::{
  grammar::{
    identifier::{CallTarget, FullIdentifier, Identifier, Name, Type},
    operators::{AssignOp, BinOp, UnOp}, semifier::resolver::NameResolver,
  },
  report::location::WithLineInfo,
};

use super::attributes::Attribute;

#[derive(Debug, Clone, PartialEq)]
pub struct TypedNameWithLineInfo {
  pub name: WithLineInfo<Name>,
  pub typ: WithLineInfo<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedName {
  pub name: Name,
  pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct OptionalTypedNameWithLineInfo {
  pub name: WithLineInfo<Name>,
  pub typ: Option<WithLineInfo<Type>>,
}

#[derive(Debug, Clone)]
pub struct OptionalTypedName {
  pub name: Name,
  pub typ: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum Expression<I: Clone> {
  AtomBoolean(WithLineInfo<bool>),
  AtomInteger(WithLineInfo<isize>),
  AtomFloat(WithLineInfo<f64>),
  AtomString(WithLineInfo<String>),
  AtomIdentifier(I),

  Call(WithLineInfo<CallTarget<I>>, Vec<Expression<I>>),

  UnOp(WithLineInfo<UnOp>, Box<Expression<I>>),
  BinOp(Box<Expression<I>>, WithLineInfo<BinOp>, Box<Expression<I>>),
}

#[derive(Debug, Clone)]
pub enum Node {
  Expression(Expression<Identifier>),
  Assignment {
    target: Identifier,
    op: WithLineInfo<AssignOp>,
    val: Expression<Identifier>,
  },
  VarDecl {
    typ: OptionalTypedNameWithLineInfo,
    val: Expression<Identifier>,
    mutable: bool,
  },
  FnDecl {
    attributes: Vec<WithLineInfo<Attribute>>,
    name: WithLineInfo<Name>,
    params: Vec<TypedNameWithLineInfo>,
    ret_type: Option<WithLineInfo<Type>>,
    body: Vec<Node>,
  },
  Return(Expression<Identifier>),
  ModDecl(WithLineInfo<Name>),
  UseDecl(Identifier),
  StructDecl {
    name: WithLineInfo<Name>,
    fields: Vec<TypedNameWithLineInfo>,
  },
}

impl<I: Clone> Expression<I> {
  pub fn dependencies(&self) -> Vec<I> {
    match self {
      Expression::AtomIdentifier(id) => vec![id.clone()],
      Expression::BinOp(left, _, right) => {
        let mut ids = left.dependencies();
        ids.extend(right.dependencies());
        ids
      }
      Expression::UnOp(_, expr) => expr.dependencies(),
      Expression::Call(target, args) => {
        let mut ids = if let CallTarget::Declared(id) = &target.value {
          vec![id.clone()]
        } else {
          vec![]
        };
        ids.extend(args.iter().flat_map(|arg| arg.dependencies()));
        ids
      }
      _ => vec![],
    }
  }

  pub fn first_call(&self) -> Option<WithLineInfo<CallTarget<I>>> {
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

impl TypedNameWithLineInfo {
  pub fn unwrap(self) -> TypedName {
    TypedName {
      name: self.name.unwrap(),
      typ: self.typ.unwrap(),
    }
  }
}

impl Expression<Identifier> {
  pub fn resolve(self, resolver: &NameResolver) -> Expression<FullIdentifier> {
    match self {
      Expression::AtomIdentifier(id) => {
        let full_id = resolver.resolve(&id);
        Expression::AtomIdentifier(full_id)
      }
      Expression::Call(target, args) => {
        let full_target = match &target.value {
          CallTarget::Declared(id) => CallTarget::Declared(resolver.resolve(&id)),
          CallTarget::Builtin(name) => CallTarget::Builtin(name.clone()),
        };
        let full_args = args.into_iter().map(|arg| arg.resolve(resolver)).collect();
        Expression::Call(target.map(|_| full_target), full_args)
      }
      Expression::UnOp(op, expr) => {
        let full_expr = expr.resolve(resolver);
        Expression::UnOp(op, Box::new(full_expr))
      }
      Expression::BinOp(left, op, right) => {
        let full_left = left.resolve(resolver);
        let full_right = right.resolve(resolver);
        Expression::BinOp(Box::new(full_left), op, Box::new(full_right))
      }
      Expression::AtomBoolean(b) => Expression::AtomBoolean(b),
      Expression::AtomInteger(i) => Expression::AtomInteger(i),
      Expression::AtomFloat(f) => Expression::AtomFloat(f),
      Expression::AtomString(s) => Expression::AtomString(s),
    }
  }
}

impl OptionalTypedNameWithLineInfo {
  pub fn unwrap(self) -> OptionalTypedName {
    OptionalTypedName {
      name: self.name.unwrap(),
      typ: self.typ.map(|t| t.unwrap()),
    }
  }
}
