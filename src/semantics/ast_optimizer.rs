use crate::grammar::parser::{ExpressionNode, Node};

fn expression_optimize(expression: &mut ExpressionNode) {
  println!("{:?}", expression);
  if expression.can_reduce() {
    expression.reduce();
  } else {
    match expression {
      ExpressionNode::BinOp(left, _, right) => {
        expression_optimize(left);
        expression_optimize(right);
      }
      ExpressionNode::UnOp(_, expr) => {
        expression_optimize(expr);
      }
      ExpressionNode::FunctionCall(_, exprs) => {
        for expr in exprs {
          expression_optimize(expr);
        }
      }
      ExpressionNode::BuiltinCall(_, exprs) => {
        for expr in exprs {
          expression_optimize(expr);
        }
      }
      _ => {}
    }
  }
}

pub fn ast_optimize(nodes: &mut Vec<Node>) {
  for node in nodes {
    match node {
      Node::Expression(expr) => expression_optimize(expr),
      Node::Assignment(_, _, expr) => expression_optimize(expr),
      Node::FnDecl(_, _, _, nodes) => ast_optimize(nodes),
      _ => {}
    }
  }
}
