use super::{
  builtins::{Builtin, BuiltinFn, BuiltinType},
  identifier::{Identifier, Name, Type, TypedName},
  keywords::Keyword,
  lexer::{Token, TokenMeta},
  operators::{AssignOp, BinOp, Precedence, UnOp},
};
use crate::report::error::report_and_exit;
use peg::{error::ExpectedSet, parser};
use std::path::PathBuf;

parser! {
  grammar parser<'a>() for [&'a Token] {
    rule name() -> Name = quiet! {
      [Token::Identifier(name) if name.is_singular()] {
        name.parts[0].clone()
    }} / expected!("Name");

    rule param_decl() -> TypedName = quiet! {
      name:name() [Token::Separator]? [Token::Colon] [Token::Separator]?
      tn:(
        [Token::Identifier(t)] {
          TypedName(name.clone(), Type::Declared(t.clone()))
        } /
        [Token::Builtin(Builtin::Type(t))] {
          TypedName(name.clone(), Type::Builtin(*t))
        }
      ) { tn }
    } / expected!("Parameter");
    rule param_sep() = quiet! {
      [Token::Separator]? [Token::Comma] [Token::Separator]?
    } / expected!("Parameter Separator");
    rule params_decl() -> Vec<TypedName> = quiet! {
      p:(
        param_decl() ** param_sep()
      ) param_sep()? { p }
    } / expected!("Parameters");

    rule params() -> Vec<RawExpressionNode> = quiet! {
      p:(
        expression() ** param_sep()
      ) param_sep()? { p }
    } / expected!("Arguments");

    rule _() = [Token::Separator]?

    #[cache_left_rec]
    rule expression() -> RawExpressionNode =
    start:position!()
    data:precedence! {
        x:expression() _ [Token::Op(op) if op.binary_with(Precedence::Lowest)] _ y:expression() {
          ExpressionNodeData::BinOp(Box::new(x), op.as_binary(), Box::new(y))
        }
        --
        x:expression() _ [Token::Op(op) if op.binary_with(Precedence::Low)] _ y:expression() {
          ExpressionNodeData::BinOp(Box::new(x), op.as_binary(), Box::new(y))
        }
        --
        x:expression() _ [Token::Op(op) if op.binary_with(Precedence::High)] _ y:expression() {
          ExpressionNodeData::BinOp(Box::new(x), op.as_binary(), Box::new(y))
        }
        --
        [Token::Op(op) if op.can_be_unary()] _ x:expression() {
          ExpressionNodeData::<RawExpressionNode>::UnOp(op.as_unary(), Box::new(x))
        }
        --
        [Token::LiteralInteger(i)] { ExpressionNodeData::AtomInteger(*i as isize) }
        [Token::LiteralFloat(f)] { ExpressionNodeData::AtomFloat(*f) }
        [Token::LiteralBoolean(b)] { ExpressionNodeData::AtomBoolean(*b) }
        [Token::LiteralString(s)] { ExpressionNodeData::AtomString(s.clone()) }
        [Token::Identifier(name)] { ExpressionNodeData::AtomIdentifier(name.clone()) }
        en:(
          [Token::Builtin(Builtin::Fn(b))] _
          [Token::ParenOpen] _
          args:params() _
          [Token::ParenClose] _ { ExpressionNodeData::BuiltinCall(*b, args) }
        ) { en }
        en:(
          [Token::Identifier(name)] _
          [Token::ParenOpen] _
          args:params() _
          [Token::ParenClose] _ {
            ExpressionNodeData::FunctionCall(name.clone(), args)
          }
        ) { en }
        [Token::ParenOpen] _ x:expression()  _ [Token::ParenClose] { x.data }
    }
    end:position!() {
      RawExpressionNode {
        data: data,
        start: start,
        len: end - start
      }
    };

    rule assignment() -> RawNode =
    start:position!()
    [Token::Identifier(name)] [Token::Separator]?
    [Token::AssignOp(op)] [Token::Separator]?
    e:expression()
    end:position!() {
      RawNode {
        data: NodeData::Assignment(name.clone(), *op, e),
        start: start,
        len: end - start
      }
    };
    rule var_decl() -> RawNode =
    start:position!()
      [Token::Keyword(Keyword::Let)] [Token::Separator]
      name:name() [Token::Separator]?
      tn:([Token::Colon] [Token::Separator]?
        tn:(
          [Token::Identifier(t)] {
            TypedName(name.clone(), Type::Declared(t.clone()))
          } /
          [Token::Builtin(Builtin::Type(t))] {
            TypedName(name.clone(), Type::Builtin(*t))
          }
        )? { tn }) [Token::Separator]?
      [Token::AssignOp(AssignOp::Identity)] [Token::Separator]?
      e:expression()
      end:position!() {
        RawNode {
          data: NodeData::VarDecl(tn, e),
          start: start,
          len: end - start
        }
      };

    rule statement() -> RawNode = quiet! {
      start:position!() e:expression() end:position!() {
        RawNode {
          data: NodeData::Expression(e),
          start,
          len: end - start,
        }
      } /
      assignment() /
      var_decl()
    } / expected!("Statement");

    rule statement_sep() = quiet! {
      [Token::Separator]? [Token::SemiColon] [Token::Separator]?
    } / expected!("Statement Separator");

    rule statements() -> Vec<RawNode> = quiet! {
      s:(
        statement() ** statement_sep()
      ) statement_sep()? { s }
    } / expected!("Statements");

    rule global_fn_decl() -> RawNode =
      start:position!()
      [Token::Keyword(Keyword::Fn)] [Token::Separator]
      name:name() [Token::Separator]?
      [Token::ParenOpen] [Token::Separator]?
      params:params_decl() [Token::Separator]?
      [Token::ParenClose] [Token::Separator]?
      ret_type:(
        [Token::Arrow] [Token::Separator]? t:(
          [Token::Identifier(t)] { Type::Declared(t.clone()) } /
          [Token::Builtin(Builtin::Type(t))] { Type::Builtin(*t) }
        ) [Token::Separator]? { t }

      )?
      [Token::BraceOpen] [Token::Separator]?
      stmts:statements() [Token::Separator]?
      [Token::BraceClose]
      end:position!() {
        RawNode {
          data: NodeData::FnDecl(
            name,
            params,
            ret_type.unwrap_or(
              Type::Builtin(
                BuiltinType::Void
              )
            ),
            stmts
          ),
          start: start,
          len: end - start
        }
      };

    rule global_var_decl() -> RawNode =
      d:var_decl() statement_sep() { d };

    rule global_mod_decl() -> RawNode =
      start:position!()
      [Token::Keyword(Keyword::Mod)] [Token::Separator]
      name:name() statement_sep()
      end:position!() {
        RawNode {
          data: NodeData::ModDecl(name),
          start,
          len: end - start,
        }
      };

    rule global_struct_decl() -> RawNode =
      start:position!()
      [Token::Keyword(Keyword::Struct)] [Token::Separator]
      name:name() [Token::Separator]?
      [Token::BraceOpen] [Token::Separator]?
      fields:params_decl() [Token::Separator]?
      [Token::BraceClose]
      end:position!() {
        RawNode {
          data: NodeData::StructDecl(name, fields),
          start,
          len: end - start,
        }
      };

    rule decl() -> RawNode =
      global_fn_decl() /
      global_var_decl() /
      global_mod_decl() /
      global_struct_decl()
;

    pub(super) rule decls() -> Vec<RawNode> =
      decl() ** ([Token::Separator]?)
  }
}

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
pub enum NodeData<E, D> {
  Expression(E),
  Assignment(Identifier, AssignOp, E),
  VarDecl(Option<TypedName>, E),
  FnDecl(Name, Vec<TypedName>, Type, Vec<D>),
  ModDecl(Name),
  StructDecl(Name, Vec<TypedName>),
}

#[derive(Debug, Clone)]
struct RawExpressionNode {
  data: ExpressionNodeData<RawExpressionNode>,
  start: usize,
  len: usize,
}

#[derive(Debug, Clone)]
struct RawNode {
  data: NodeData<RawExpressionNode, RawNode>,
  start: usize,
  len: usize,
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
  pub data: NodeData<ExpressionNode, Node>,
  pub line: usize,
  pub column: usize,
  pub len: usize,
}

pub struct Parser;

#[derive(Debug, Clone)]
pub struct ParserError {
  pub line: usize,
  pub column: usize,
  pub len: usize,
  pub unexpected: Token,
  pub expected: ExpectedSet,
}

fn get_line_info(
  start: usize,
  len: usize,
  tokens: &[TokenMeta],
) -> (usize, usize, usize) {
  let line = tokens[start].line;
  let column = tokens[start].column;
  let len = tokens[start..start + len].iter().map(|tm| tm.len).sum();
  (line, column, len)
}

fn bake_expression_data(
  raw_expression_data: ExpressionNodeData<RawExpressionNode>,
  tokens: &[TokenMeta],
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
  tokens: &[TokenMeta],
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
  raw_node_data: NodeData<RawExpressionNode, RawNode>,
  tokens: &[TokenMeta],
) -> NodeData<ExpressionNode, Node> {
  match raw_node_data {
    NodeData::Expression(expr) => {
      NodeData::Expression(bake_expression(expr, tokens))
    }
    NodeData::Assignment(id, op, expr) => {
      NodeData::Assignment(id, op, bake_expression(expr, tokens))
    }
    NodeData::VarDecl(tn, expr) => {
      NodeData::VarDecl(tn, bake_expression(expr, tokens))
    }
    NodeData::FnDecl(name, params, ret_type, stmts) => {
      let baked_stmts = stmts
        .into_iter()
        .map(|stmt| bake_node(stmt, tokens))
        .collect();
      NodeData::FnDecl(name, params, ret_type, baked_stmts)
    }
    NodeData::ModDecl(name) => NodeData::ModDecl(name),
    NodeData::StructDecl(name, fields) => NodeData::StructDecl(name, fields),
  }
}
fn bake_node(raw_node: RawNode, tokens: &[TokenMeta]) -> Node {
  let (line, column, len) = get_line_info(raw_node.start, raw_node.len, tokens);
  Node {
    data: bake_node_data(raw_node.data, tokens),
    line,
    column,
    len,
  }
}

impl Parser {
  pub fn parse(&self, tokens: &[TokenMeta]) -> Result<Vec<Node>, ParserError> {
    let tokens_ref = tokens.iter().map(|tm| &tm.token).collect::<Vec<_>>();
    match parser::decls(&tokens_ref) {
      Ok(nodes) => {
        Ok(nodes.into_iter().map(|rn| bake_node(rn, tokens)).collect())
      }
      Err(e) => Err(ParserError {
        line: tokens[e.location].line,
        column: tokens[e.location].column,
        len: tokens[e.location].len,
        unexpected: tokens[e.location].token.clone(),
        expected: e.expected,
      }),
    }
  }
}

impl ParserError {
  pub fn report_and_exit(&self, path: &PathBuf, source: &str) -> ! {
    let line = source.lines().nth(self.line - 1).unwrap();

    let message =
      format!("Unexpecected token: {}", self.unexpected.error_symbol());
    let expected_count = self.expected.tokens().count();
    let expected_list = self.expected.tokens().collect::<Vec<_>>().join(", ");

    let expected = if expected_count > 1 {
      format!("Expected one of: {}", expected_list)
    } else {
      format!("Expected: {}", expected_list)
    };

    report_and_exit(
      line,
      path,
      self.line,
      self.column,
      self.len,
      &message,
      Some(&expected),
      1,
    )
  }
}
