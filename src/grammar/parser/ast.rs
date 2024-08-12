use crate::{
  grammar::{
    builtins::BuiltinFn,
    identifier::{
      Identifier, Name, Type, TypedNameWithLineInfo, TypedNameWithRawLineInfo,
    },
    operators::{AssignOp, BinOp, UnOp},
  },
  report::location::{WithLineInfo, WithRawLineInfo},
};

#[derive(Debug, Clone)]
pub enum ExpressionNodeData<E> {
  AtomInteger(isize),
  AtomString(String),
  AtomBoolean(bool),
  AtomFloat(f64),
  AtomIdentifier(Identifier),

  BuiltinCall(BuiltinFn, Vec<E>),
  FunctionCall(Identifier, Vec<E>),

  UnOp(UnOp, Box<E>),
  BinOp(Box<E>, BinOp, Box<E>),
}

#[derive(Debug, Clone)]
pub enum NodeData<Expr, Node, Identifier, Name, TypedName, Type, AssignOp> {
  Expression(Expr),
  Assignment(Identifier, AssignOp, Expr),
  VarDecl(Option<TypedName>, Expr),
  FnDecl(Name, Vec<TypedName>, Option<Type>, Vec<Node>),
  ModDecl(Name),
  StructDecl(Name, Vec<TypedName>),
}

#[derive(Debug, Clone)]
pub(super) struct RawExpressionNode {
  pub data: ExpressionNodeData<RawExpressionNode>,
  pub start: usize,
  pub len: usize,
}

#[derive(Debug, Clone)]
pub(super) struct RawNode {
  pub data: RawNodeData,
  pub start: usize,
  pub len: usize,
}

#[derive(Debug, Clone)]
pub struct ExpressionNode {
  pub data: ExpressionNodeData<ExpressionNode>,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
  pub data: BakedNodeData,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

pub type BakedNodeData = NodeData<
  ExpressionNode,
  Node,
  WithLineInfo<Identifier>,
  WithLineInfo<Name>,
  TypedNameWithLineInfo,
  WithLineInfo<Type>,
  WithLineInfo<AssignOp>,
>;
pub(super) type RawNodeData = NodeData<
  RawExpressionNode,
  RawNode,
  WithRawLineInfo<Identifier>,
  WithRawLineInfo<Name>,
  TypedNameWithRawLineInfo,
  WithRawLineInfo<Type>,
  WithRawLineInfo<AssignOp>,
>;
