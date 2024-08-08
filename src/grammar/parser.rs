use super::{
  builtins::{Builtin, BuiltinFn, BuiltinType},
  identifier::{Identifier, Name, Type, TypedName},
  keywords::Keyword,
  lexer::Token,
  operators::{AssignOp, BinOp, Precedence, UnOp},
};
use peg::parser;

parser! {
  pub grammar parser<'a>() for [&'a Token] {
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

    rule params() -> Vec<ExpressionNode> = quiet! {
      p:(
        expression() ** param_sep()
      ) param_sep()? { p }
    } / expected!("Arguments");

    rule optsep() = [Token::Separator]?

    rule expression() -> ExpressionNode = precedence! {
      x:(@) optsep() [Token::Op(op) if op.binary_with(Precedence::Lowest)] optsep() y:@ {
        ExpressionNode::BinOp(Box::new(x), op.as_binary(), Box::new(y))
      }
      --
      x:(@) optsep() [Token::Op(op) if op.binary_with(Precedence::Low)] optsep() y:@ {
        ExpressionNode::BinOp(Box::new(x), op.as_binary(), Box::new(y))
      }
      --
      x:(@) optsep() [Token::Op(op) if op.binary_with(Precedence::High)] optsep() y:@ {
        ExpressionNode::BinOp(Box::new(x), op.as_binary(), Box::new(y))
      }
      --
      [Token::Op(op) if op.can_be_unary()] optsep() x:@ {
        ExpressionNode::UnOp(op.as_unary(), Box::new(x))
      }
      --
      [Token::LiteralInteger(i)] { ExpressionNode::AtomInteger(*i as isize) }
      [Token::LiteralFloat(f)] { ExpressionNode::AtomFloat(*f) }
      [Token::LiteralBoolean(b)] { ExpressionNode::AtomBoolean(*b) }
      [Token::LiteralString(s)] { ExpressionNode::AtomString(s.clone()) }
      [Token::Identifier(name)] { ExpressionNode::AtomIdentifier(name.clone()) }
      e:(
        [Token::ParenOpen] optsep()
        e:expression() optsep()
        [Token::ParenClose] { e }
      ) { e }
      en:(
        [Token::Builtin(Builtin::Fn(b))] optsep()
        [Token::ParenOpen] optsep()
        args:params() optsep()
        [Token::ParenClose] optsep() { ExpressionNode::BuiltinCall(*b, args) }
      ) { en }
      en:(
        [Token::Identifier(name)] optsep()
        [Token::ParenOpen] optsep()
        args:params() optsep()
        [Token::ParenClose] optsep() {
          ExpressionNode::FunctionCall(name.clone(), args)
        }
      ) { en }
      [Token::ParenOpen] optsep() x:expression()  optsep() [Token::ParenClose] { x }
    };

    rule assignment() -> Node =
    [Token::Identifier(name)] [Token::Separator]?
    [Token::AssignOp(op)] [Token::Separator]?
    e:expression() {
      Node::Assignment(name.clone(), *op, e)
    };
    rule var_decl() -> Node =
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
      e:expression() {
        Node::VarDecl(tn, e)
      };

    rule statement() -> Node = quiet! {
      e:expression() { Node::Expression(e) } /
      assignment() /
      var_decl()
    } / expected!("Statement");

    rule statement_sep() = quiet! {
      [Token::Separator]? [Token::SemiColon] [Token::Separator]?
    } / expected!("Statement Separator");

    rule statements() -> Vec<Node> = quiet! {
      s:(
        statement() ** statement_sep()
      ) statement_sep()? { s }
    } / expected!("Statements");

    rule global_fn_decl() -> Node =
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
      [Token::BraceClose] {
        Node::FnDecl(
          name,
          params,
          ret_type.unwrap_or(
            Type::Builtin(
              BuiltinType::Void
            )
          ),
          stmts
        )
      };

    rule global_var_decl() -> Node =
      d:var_decl() statement_sep() { d };

    rule global_mod_decl() -> Node =
      [Token::Keyword(Keyword::Mod)] [Token::Separator]
      name:name() statement_sep() {
        Node::ModDecl(name)
      };

    rule global_struct_decl() -> Node =
      [Token::Keyword(Keyword::Struct)] [Token::Separator]
      name:name() [Token::Separator]?
      [Token::BraceOpen] [Token::Separator]?
      fields:params_decl() [Token::Separator]?
      [Token::BraceClose] {
        Node::StructDecl(name, fields)
      };

    pub rule decl() -> Vec<Node> =
      (
        global_fn_decl() /
        global_var_decl() /
        global_mod_decl() /
        global_struct_decl()
      ) ** ([Token::Separator]?)
  }
}

#[derive(Debug, Clone)]
pub enum ExpressionNode {
  AtomInteger(isize),
  AtomString(String),
  AtomBoolean(bool),
  AtomFloat(f64),
  AtomIdentifier(Identifier),

  BuiltinCall(BuiltinFn, Vec<ExpressionNode>),
  FunctionCall(Identifier, Vec<ExpressionNode>),

  UnOp(UnOp, Box<ExpressionNode>),
  BinOp(Box<ExpressionNode>, BinOp, Box<ExpressionNode>),
}

#[derive(Debug, Clone)]
pub enum Node {
  Expression(ExpressionNode),
  Assignment(Identifier, AssignOp, ExpressionNode),
  VarDecl(Option<TypedName>, ExpressionNode),
  FnDecl(Name, Vec<TypedName>, Type, Vec<Node>),
  ModDecl(Name),
  StructDecl(Name, Vec<TypedName>),
}

impl ExpressionNode {
  pub fn can_reduce(&self) -> bool {
    match self {
      ExpressionNode::AtomInteger(_) => false,
      ExpressionNode::AtomString(_) => false,
      ExpressionNode::AtomBoolean(_) => false,
      ExpressionNode::AtomFloat(_) => false,
      ExpressionNode::AtomIdentifier(_) => false,
      ExpressionNode::BuiltinCall(_, _) => false,
      ExpressionNode::FunctionCall(_, _) => false,
      ExpressionNode::UnOp(_, operand) => operand.is_known(),
      ExpressionNode::BinOp(left, _, right) => {
        left.is_known() && right.is_known()
      }
    }
  }

  pub fn is_known(&self) -> bool {
    match self {
      ExpressionNode::AtomInteger(_) => true,
      ExpressionNode::AtomString(_) => true,
      ExpressionNode::AtomBoolean(_) => true,
      ExpressionNode::AtomFloat(_) => true,
      ExpressionNode::AtomIdentifier(_) => false,
      ExpressionNode::BuiltinCall(_, _) => false,
      ExpressionNode::FunctionCall(_, _) => false,
      ExpressionNode::UnOp(_, expr) => expr.is_known(),
      ExpressionNode::BinOp(left, _, right) => {
        left.is_known() && right.is_known()
      }
    }
  }

  pub fn reduce(&mut self) {
    match self {
      ExpressionNode::UnOp(op, expr) => {
        expr.reduce();
        match (op, *expr.clone()) {
          (UnOp::Negate, ExpressionNode::AtomInteger(i)) => {
            *self = ExpressionNode::AtomInteger(-i);
          }
          (UnOp::Negate, ExpressionNode::AtomFloat(f)) => {
            *self = ExpressionNode::AtomFloat(-f);
          }
          (UnOp::Not, ExpressionNode::AtomBoolean(b)) => {
            *self = ExpressionNode::AtomBoolean(!b);
          }
          (UnOp::Identity, _) => {
            *self = *expr.clone();
          }
          _ => {}
        }
      }
      ExpressionNode::BinOp(left, op, right) => {
        left.reduce();
        right.reduce();
        if left.is_known() && right.is_known() {
          match (*left.clone(), *right.clone()) {
            (
              ExpressionNode::AtomInteger(l),
              ExpressionNode::AtomInteger(r),
            ) if op.can_execute() => {
              *self = ExpressionNode::AtomInteger(op.execute(l, r));
            }
            (ExpressionNode::AtomFloat(l), ExpressionNode::AtomFloat(r))
              if op.can_execute() =>
            {
              *self = ExpressionNode::AtomFloat(op.execute(l, r));
            }
            (ExpressionNode::AtomInteger(l), ExpressionNode::AtomFloat(r))
              if op.can_execute() =>
            {
              *self = ExpressionNode::AtomFloat(op.execute(l as f64, r));
            }
            (ExpressionNode::AtomFloat(l), ExpressionNode::AtomInteger(r))
              if op.can_execute() =>
            {
              *self = ExpressionNode::AtomFloat(op.execute(l, r as f64));
            }
            (
              ExpressionNode::AtomBoolean(l),
              ExpressionNode::AtomBoolean(r),
            ) if op.applies_to_bool() => {
              *self = ExpressionNode::AtomBoolean(op.execute_comp(l, r));
            }
            (
              ExpressionNode::AtomInteger(l),
              ExpressionNode::AtomInteger(r),
            ) if op.is_comp() => {
              *self = ExpressionNode::AtomBoolean(op.execute_comp(l, r));
            }
            (ExpressionNode::AtomFloat(l), ExpressionNode::AtomFloat(r))
              if op.is_comp() =>
            {
              *self = ExpressionNode::AtomBoolean(op.execute_comp(l, r));
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
  }
}
