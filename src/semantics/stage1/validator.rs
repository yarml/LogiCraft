use crate::{
  error::context::ErrorContext,
  grammar::{
    operators::{BinOp, UnOp},
    parser::{ExpressionNode, Node},
  },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StaticType {
  Invalid,
  Void,
  Bool,
  Int,
  Float,
  Char,
  String,
  Any,
}

impl StaticType {
  pub fn modified_type_bin(&self, other: StaticType, op: BinOp) -> StaticType {
    if other == StaticType::Invalid
      || *self == StaticType::Invalid
      || other == StaticType::Void
      || *self == StaticType::Void
    {
      return StaticType::Invalid;
    }
    if op.is_comp() && other == *self {
      match other {
        StaticType::Int => return StaticType::Bool,
        StaticType::Float => return StaticType::Bool,
        StaticType::Char => return StaticType::Bool,
        StaticType::String => return StaticType::Bool,
        StaticType::Bool if op.applies_to_bool() => return StaticType::Bool,
        _ => return StaticType::Invalid,
      }
    }
    if *self == StaticType::Any || other == StaticType::Any {
      return StaticType::Any;
    }
    if *self == StaticType::String || other == StaticType::String {
      return StaticType::String;
    }
    if *self == StaticType::Bool || other == StaticType::Bool {
      return StaticType::Invalid;
    }
    if *self == StaticType::Char && other == StaticType::Char {
      return StaticType::Char;
    }
    if *self == StaticType::Char || other == StaticType::Char {
      return StaticType::Invalid;
    }
    if *self == StaticType::Float || other == StaticType::Float {
      return StaticType::Float;
    }
    return StaticType::Int;
  }

  pub fn modified_type_un(&self, op: UnOp) -> StaticType {
    match op {
      UnOp::Not if *self == StaticType::Bool || *self == StaticType::Any => {
        StaticType::Bool
      }
      UnOp::Identity if self.arithmetic() => *self,
      UnOp::Negate if self.arithmetic() => *self,
      _ => StaticType::Invalid,
    }
  }

  pub fn arithmetic(&self) -> bool {
    matches!(self, StaticType::Int | StaticType::Float | StaticType::Any)
  }
}

fn validate_expression(
  expression: &ExpressionNode,
  context: &mut ErrorContext,
) {
  match expression {
    ExpressionNode::BinOp(left, _, right) => {
      validate_expression(left, context);
      validate_expression(right, context);
    }
    ExpressionNode::UnOp(op, expr) => {
      validate_expression(expr, context);
    }
    ExpressionNode::BuiltinCall(_, exprs)
    | ExpressionNode::FunctionCall(_, exprs) => {
      for expr in exprs {
        validate_expression(expr, context);
      }
    }
    _ => {}
  }
}

pub fn validate(nodes: &[Node]) -> ErrorContext {
  let mut context = ErrorContext::new();
  for node in nodes {
    match node {
      Node::Expression(expr) => {
        validate_expression(expr, &mut context);
      }
      Node::Assignment(_, _, expr) => {
        validate_expression(expr, &mut context);
      }
      Node::VarDecl(_, expr) => {
        validate_expression(expr, &mut context);
      }
      Node::FnDecl(_, _, _, nodes) => {
        context.merge(validate(nodes));
      }
      _ => {}
    }
  }

  context
}
