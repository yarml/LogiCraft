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

    rule expression_atom() -> ExpressionNode =
      [Token::LiteralInteger(i)] { ExpressionNode::AtomInteger(*i as isize) } /
      [Token::LiteralFloat(f)] { ExpressionNode::AtomFloat(*f) } /
      [Token::LiteralBoolean(b)] { ExpressionNode::AtomBoolean(*b) } /
      [Token::LiteralString(s)] { ExpressionNode::AtomString(s.clone()) } /
      [Token::Identifier(name)] { ExpressionNode::AtomIdentifier(name.clone()) } /
      (
        [Token::ParenOpen] optsep()
        e:expression() optsep()
        [Token::ParenClose] { e }
      ) /
      (
        [Token::Builtin(Builtin::Fn(b))] optsep()
        [Token::ParenOpen] optsep()
        args:params() optsep()
        [Token::ParenClose] optsep()
        { ExpressionNode::BuiltinCall(*b, args) }
      ) /
      (
        [Token::Identifier(name)] optsep()
        [Token::ParenOpen] optsep()
        args:params() optsep()
        [Token::ParenClose] optsep() {
          ExpressionNode::FunctionCall(name.clone(), args)
        }
      );

    rule expression_unop() -> ExpressionNode =
      [Token::Op(op) if op.can_be_unary()] e:expression_atom() {
        ExpressionNode::UnOp(op.as_unary(), Box::new(e))
      } / expression_atom();

    rule expression_factor() -> ExpressionNode =
      e1:expression_unop() optsep()
      [Token::Op(op) if op.binary_with(Precedence::High)] optsep()
      e2:expression_unop() {
        ExpressionNode::BinOp(Box::new(e1), op.as_binary(), Box::new(e2))
      } / expression_unop();
  
    rule expression_term() -> ExpressionNode =
      e1:expression_factor() optsep()
      [Token::Op(op) if op.binary_with(Precedence::Low)] optsep()
      e2:expression_factor() {
        ExpressionNode::BinOp(Box::new(e1), op.as_binary(), Box::new(e2))
      } / expression_factor();
    
    rule expression_comp() -> ExpressionNode =
      e1:expression_term() optsep()
      [Token::Op(op) if op.binary_with(Precedence::Lowest)] optsep()
      e2:expression_term() {
        ExpressionNode::BinOp(Box::new(e1), op.as_binary(), Box::new(e2))
      } / expression_term();

    rule expression() -> ExpressionNode = expression_comp();

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
