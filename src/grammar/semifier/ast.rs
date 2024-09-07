use crate::{
  grammar::{
    builtins::BuiltinType,
    identifier::{FullIdentifier, Name, Type},
    operators::AssignOp,
    parser::{
      ast::{Expression, Node, OptionalTypedName, TypedName},
      attributes::Attribute,
    },
  },
  report::message::Message,
};

use super::resolver::NameResolver;

#[derive(Debug, Clone)]
pub struct FnDecl {
  pub attributes: Vec<Attribute>,
  pub name: Name,
  pub params: Vec<TypedName>,
  pub ret_type: Type,
  pub dependencies: Vec<FullIdentifier>,
  pub locals: Vec<OptionalTypedName>,
  pub body: Vec<FnStatement>,
}

#[derive(Debug, Clone)]
pub enum FnStatement {
  Assignment {
    target: FullIdentifier,
    op: AssignOp,
    val: Expression<FullIdentifier>,
  },
  SideEffect(Expression<FullIdentifier>),
  Return(Expression<FullIdentifier>),
}

impl FnDecl {
  pub fn from_function_node(node: Node, resolver: &mut NameResolver) -> Self {
    if let Node::FnDecl {
      attributes,
      name,
      params,
      ret_type,
      body,
    } = node
    {
      resolver.push_scope();
      let mut locals = Vec::new();
      let mut deps = Vec::new();
      let mut minbody = Vec::new();
      for node in body {
        match node {
          Node::VarDecl { typ, val, .. } => {
            let name = typ.name.clone().unwrap();
            let resolved_expr = val.resolve(resolver);
            deps.extend_from_slice(&resolved_expr.dependencies());
            resolver.decl_local(name.clone());
            locals.push(typ.unwrap());

            minbody.push(FnStatement::Assignment {
              target: FullIdentifier::Local(name),
              op: AssignOp::Identity,
              val: resolved_expr,
            });
          }
          Node::Assignment { target, op, val } => {
            let resolved_target = resolver.resolve(&target);
            let resolved_val = val.resolve(resolver);
            deps.push(resolved_target.clone());
            deps.extend_from_slice(&resolved_val.dependencies());
            minbody.push(FnStatement::Assignment {
              target: resolved_target,
              op: op.unwrap(),
              val: resolved_val,
            });
          }
          Node::Return(expr) => {
            let resolved_expr = expr.resolve(resolver);
            deps.extend_from_slice(&resolved_expr.dependencies());
            minbody.push(FnStatement::Return(resolved_expr));
          }
          _ => Message::compiler_bug("Unexpected node in function body")
            .report_and_exit(1),
        };
      }
      resolver.pop_scope();
      Self {
        attributes: attributes.into_iter().map(|att| att.unwrap()).collect(),
        name: name.unwrap(),
        params: params.into_iter().map(|param| param.unwrap()).collect(),
        ret_type: ret_type
          .map(|ret| ret.unwrap())
          .unwrap_or_else(|| Type::Builtin(BuiltinType::Void)),
        dependencies: deps,
        locals,
        body: minbody,
      }
    } else {
      Message::compiler_bug("Expected FnDecl node").report_and_exit(1)
    }
  }
}
