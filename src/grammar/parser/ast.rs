use crate::{
  grammar::{
    builtins::BuiltinType,
    identifier::{
      CallTarget, Name, ScopedIdentifier, Type, UnscopedIdentifier,
    },
    operators::{AssignOp, BinOp, UnOp},
    semantics::decl::ScopedNameResolver,
  },
  report::location::WithLineInfo,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedNameLI<I: Clone> {
  pub name: WithLineInfo<Name>,
  pub typ: WithLineInfo<Type<I>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedName<I: Clone> {
  pub name: Name,
  pub typ: Type<I>,
}

#[derive(Debug, Clone)]
pub struct OptionalTypedNameLI<I: Clone> {
  pub name: WithLineInfo<Name>,
  pub typ: Option<WithLineInfo<Type<I>>>,
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
  Expression(Expression<UnscopedIdentifier>),
  Assignment {
    target: UnscopedIdentifier,
    op: WithLineInfo<AssignOp>,
    val: Expression<UnscopedIdentifier>,
  },
  VarDecl {
    typ: OptionalTypedNameLI<UnscopedIdentifier>,
    val: Expression<UnscopedIdentifier>,
    mutable: bool,
  },
  FnDecl {
    attributes: Vec<WithLineInfo<Name>>,
    name: WithLineInfo<Name>,
    params: Vec<TypedNameLI<UnscopedIdentifier>>,
    ret_type: Option<WithLineInfo<Type<UnscopedIdentifier>>>,
    body: Vec<Node>,
  },
  Return(Expression<UnscopedIdentifier>),
  ModDecl(WithLineInfo<Name>),
  UseDecl(UnscopedIdentifier),
  StructDecl {
    name: WithLineInfo<Name>,
    fields: Vec<TypedNameLI<UnscopedIdentifier>>,
  },
}

impl<I: Clone> Expression<I> {
  pub fn as_const(&self) -> Option<Expression<()>> {
    match self {
      Expression::AtomBoolean(b) => Some(Expression::AtomBoolean(b.clone())),
      Expression::AtomInteger(i) => Some(Expression::AtomInteger(i.clone())),
      Expression::AtomFloat(f) => Some(Expression::AtomFloat(f.clone())),
      Expression::AtomString(s) => Some(Expression::AtomString(s.clone())),
      Expression::AtomIdentifier(_) => None,
      Expression::Call(_, _) => None,
      Expression::UnOp(op, expr) => {
        Some(Expression::UnOp(op.clone(), Box::new(expr.as_const()?)))
      }
      Expression::BinOp(l, op, r) => Some(Expression::BinOp(
        Box::new(l.as_const()?),
        op.clone(),
        Box::new(r.as_const()?),
      )),
    }
  }

  pub fn minify(&self) -> Vec<Expression<I>> {
    match self {
      Expression::AtomBoolean(_) => vec![],
      Expression::AtomInteger(_) => vec![],
      Expression::AtomFloat(_) => vec![],
      Expression::AtomString(_) => vec![],
      Expression::AtomIdentifier(_) => vec![],
      Expression::Call(target, params) => {
        vec![Expression::Call(target.clone(), params.clone())]
      }
      Expression::UnOp(_, expr) => expr.minify(),
      Expression::BinOp(left, _, right) => {
        let mut min_left = left.minify();
        let mut min_right = right.minify();
        min_left.append(&mut min_right);
        min_left
      }
    }
  }
}

impl Expression<UnscopedIdentifier> {
  pub fn resolve(
    &self,
    resolver: &ScopedNameResolver,
  ) -> Expression<ScopedIdentifier> {
    match self {
      Expression::AtomBoolean(b) => Expression::AtomBoolean(b.clone()),
      Expression::AtomInteger(i) => Expression::AtomInteger(i.clone()),
      Expression::AtomFloat(f) => Expression::AtomFloat(f.clone()),
      Expression::AtomString(s) => Expression::AtomString(s.clone()),
      Expression::AtomIdentifier(id) => {
        Expression::AtomIdentifier(resolver.resolve(id))
      }
      Expression::Call(target, params_expr) => {
        let resolved_target = target.clone().map(|target| match target {
          CallTarget::Builtin(bfn) => CallTarget::Builtin(bfn),
          CallTarget::Declared(id) => {
            CallTarget::Declared(resolver.resolve(&id))
          }
        });
        let params_resolved = params_expr
          .iter()
          .map(|param_expr| param_expr.resolve(resolver))
          .collect();
        Expression::Call(resolved_target, params_resolved)
      }
      Expression::UnOp(op, expr) => {
        Expression::UnOp(op.clone(), Box::new(expr.resolve(resolver)))
      }
      Expression::BinOp(left, op, right) => Expression::BinOp(
        Box::new(left.resolve(resolver)),
        op.clone(),
        Box::new(right.resolve(resolver)),
      ),
    }
  }
}

impl<I: Clone> OptionalTypedNameLI<I> {
  pub fn definite(&self) -> Type<I> {
    match &self.typ {
      None => Type::Builtin(BuiltinType::Void),
      Some(typ) => typ.value.clone(),
    }
  }
}

impl<I: Clone> From<OptionalTypedNameLI<I>> for TypedNameLI<I> {
  fn from(value: OptionalTypedNameLI<I>) -> Self {
    Self {
      name: value.name,
      typ: value
        .typ
        .unwrap_or(WithLineInfo::debug(Type::Builtin(BuiltinType::Void))),
    }
  }
}
