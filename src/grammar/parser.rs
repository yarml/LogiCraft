pub mod ast;
pub mod error;
mod peg;

use super::{
  identifier::{TypedNameWithLineInfo, TypedNameWithRawLineInfo},
  lexer::token::Token,
};
use crate::report::{location::WithLineInfo, location::WithRawLineInfo};
use ast::{
  BakedNodeData, ExpressionNode, ExpressionNodeData, Node, NodeData,
  RawExpressionNode, RawNode, RawNodeData,
};
use error::ParserError;

pub struct Parser;

fn get_line_info(
  start: usize,
  len: usize,
  tokens: &[WithLineInfo<Token>],
) -> (usize, usize, usize) {
  let line = tokens[start].line;
  let column = tokens[start].column;
  let len = tokens[start..start + len].iter().map(|tm| tm.len).sum();
  (line, column, len)
}

fn bake_generic<T>(
  raw: WithRawLineInfo<T>,
  tokens: &[WithLineInfo<Token>],
) -> WithLineInfo<T> {
  let (line, column, len) = get_line_info(raw.start, raw.len, tokens);
  WithLineInfo {
    value: raw.value,
    line,
    column,
    len,
  }
}

fn bake_typed_name(
  raw_typed_name: TypedNameWithRawLineInfo,
  tokens: &[WithLineInfo<Token>],
) -> TypedNameWithLineInfo {
  TypedNameWithLineInfo(
    bake_generic(raw_typed_name.0, tokens),
    WithLineInfo {
      value: raw_typed_name.1.value,
      line: raw_typed_name.1.start,
      column: raw_typed_name.1.start,
      len: raw_typed_name.1.len,
    },
  )
}

fn bake_expression_data(
  raw_expression_data: ExpressionNodeData<RawExpressionNode>,
  tokens: &[WithLineInfo<Token>],
) -> ExpressionNodeData<ExpressionNode> {
  match raw_expression_data {
    ExpressionNodeData::AtomInteger(i) => ExpressionNodeData::AtomInteger(i),
    ExpressionNodeData::AtomString(s) => ExpressionNodeData::AtomString(s),
    ExpressionNodeData::AtomBoolean(b) => ExpressionNodeData::AtomBoolean(b),
    ExpressionNodeData::AtomFloat(f) => ExpressionNodeData::AtomFloat(f),
    ExpressionNodeData::AtomIdentifier(id) => {
      ExpressionNodeData::AtomIdentifier(id)
    }
    ExpressionNodeData::BuiltinCall(fun, params) => {
      let baked_params = params
        .into_iter()
        .map(|p| bake_expression(p, tokens))
        .collect();
      ExpressionNodeData::BuiltinCall(fun, baked_params)
    }
    ExpressionNodeData::FunctionCall(fun, params) => {
      let baked_params = params
        .into_iter()
        .map(|p| bake_expression(p, tokens))
        .collect();
      ExpressionNodeData::FunctionCall(fun, baked_params)
    }
    ExpressionNodeData::UnOp(op, expr) => {
      let baked_expr = bake_expression(*expr, tokens);
      ExpressionNodeData::UnOp(op, Box::new(baked_expr))
    }
    ExpressionNodeData::BinOp(left, op, right) => {
      let baked_left = bake_expression(*left, tokens);
      let baked_right = bake_expression(*right, tokens);
      ExpressionNodeData::BinOp(Box::new(baked_left), op, Box::new(baked_right))
    }
  }
}

fn bake_expression(
  raw_expression: RawExpressionNode,
  tokens: &[WithLineInfo<Token>],
) -> ExpressionNode {
  let (line, column, len) =
    get_line_info(raw_expression.start, raw_expression.len, tokens);

  ExpressionNode {
    data: bake_expression_data(raw_expression.data, tokens),
    line,
    column,
    len,
  }
}
fn bake_node_data(
  raw_node_data: RawNodeData,
  tokens: &[WithLineInfo<Token>],
) -> BakedNodeData {
  match raw_node_data {
    NodeData::Expression(expr) => {
      NodeData::Expression(bake_expression(expr, tokens))
    }
    NodeData::Assignment(id, op, expr) => NodeData::Assignment(
      bake_generic(id, tokens),
      bake_generic(op, tokens),
      bake_expression(expr, tokens),
    ),
    NodeData::VarDecl(tn, expr) => NodeData::VarDecl(
      tn.map(|tn| bake_typed_name(tn, tokens)),
      bake_expression(expr, tokens),
    ),
    NodeData::FnDecl(name, params, ret_type, stmts) => {
      let baked_stmts = stmts
        .into_iter()
        .map(|stmt| bake_node(stmt, tokens))
        .collect();
      let baked_params = params
        .into_iter()
        .map(|p| bake_typed_name(p, tokens))
        .collect();
      NodeData::FnDecl(
        bake_generic(name, tokens),
        baked_params,
        ret_type.map(|t| bake_generic(t, tokens)),
        baked_stmts,
      )
    }
    NodeData::ModDecl(name) => NodeData::ModDecl(bake_generic(name, tokens)),
    NodeData::StructDecl(name, fields) => {
      let baked_fields = fields
        .into_iter()
        .map(|f| bake_typed_name(f, tokens))
        .collect();
      NodeData::StructDecl(bake_generic(name, tokens), baked_fields)
    }
  }
}
fn bake_node(raw_node: RawNode, tokens: &[WithLineInfo<Token>]) -> Node {
  let (line, column, len) = get_line_info(raw_node.start, raw_node.len, tokens);
  Node {
    data: bake_node_data(raw_node.data, tokens),
    line,
    column,
    len,
  }
}

impl Parser {
  pub fn parse(
    &self,
    tokens: &[WithLineInfo<Token>],
  ) -> Result<Vec<Node>, ParserError> {
    let tokens_ref = tokens.iter().map(|tm| &tm.value).collect::<Vec<_>>();

    let line_info = |start: usize, len: usize| {
      let line = tokens[start].line;
      let column = tokens[start].column;
      let len: usize = tokens[start..start + len].iter().map(|tm| tm.len).sum();
      (line, column, len)
    };

    match peg::parser::decls(&tokens_ref) {
      Ok(nodes) => {
        Ok(nodes.into_iter().map(|rn| bake_node(rn, tokens)).collect())
      }
      Err(e) => Err(ParserError {
        line: tokens[e.location].line,
        column: tokens[e.location].column,
        len: tokens[e.location].len,
        unexpected: tokens[e.location].value.clone(),
        expected: e.expected,
      }),
    }
  }
}
