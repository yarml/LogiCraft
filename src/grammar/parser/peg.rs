use super::ast::{Expression, Node, TypedName};
use super::helper::LineInfoFn;
use crate::{
  grammar::{
    builtins::Builtin,
    identifier::{CallTarget, Identifier, Name, Type},
    keywords::Keyword,
    lexer::token::Token,
    operators::{AssignOp, BinOp, Precedence, UnOp},
  },
  report::location::WithLineInfo,
};

peg::parser! {
  pub grammar parser<'a>(line_info: &LineInfoFn) for [&'a Token] {
    // Atoms which need to save line information
    rule name() -> WithLineInfo<Name> =
      start:position!()
      [Token::Identifier(name) if name.is_singular()]
      end:position!() { line_info.tag(name.parts[0].clone(), start, end) }
    rule identifier() -> WithLineInfo<Identifier> =
      start:position!()
      [Token::Identifier(name)]
      end:position!() { line_info.tag(name.clone(), start, end) }
    rule unop() -> WithLineInfo<UnOp> =
      start:position!()
      [Token::Op(op) if op.can_be_unary()]
      end:position!() { line_info.tag(op.as_unary(), start, end) }
    rule binop(precedence: Precedence) -> WithLineInfo<BinOp> =
      start:position!()
      [Token::Op(op) if op.binary_with(precedence)]
      end:position!() { line_info.tag(op.as_binary(), start, end) }
    rule assignop() -> WithLineInfo<AssignOp> =
      start:position!()
      [Token::AssignOp(op)]
      end:position!() { line_info.tag(*op, start, end) }
    rule call_target() -> WithLineInfo<CallTarget> =
      start:position!()
      target:(
        [Token::Identifier(id)] { CallTarget::Declared(id.clone()) } /
        [Token::Builtin(Builtin::Fn(bfn))] { CallTarget::Builtin(*bfn) }
      )
      end:position!() { line_info.tag(target, start, end) }
    rule atom_boolean() -> WithLineInfo<bool> =
      start:position!()
      [Token::LiteralBoolean(value)]
      end:position!() { line_info.tag(*value, start, end) }
    rule atom_integer() -> WithLineInfo<isize> =
      start:position!()
      [Token::LiteralInteger(value)]
      end:position!() { line_info.tag(*value as isize, start, end) }
    rule atom_float() -> WithLineInfo<f64> =
      start:position!()
      [Token::LiteralFloat(value)]
      end:position!() { line_info.tag(*value, start, end) }
    rule atom_string() -> WithLineInfo<String> =
      start:position!()
      [Token::LiteralString(value)]
      end:position!() { line_info.tag(value.clone(), start, end) }
    rule typ() -> WithLineInfo<Type> =
      start:position!()
      t:(
        [Token::Identifier(typ)] { Type::Declared(typ.clone()) } /
        [Token::Builtin(Builtin::Type(btype))] { Type::Builtin(*btype) }
      )
      end:position!() { line_info.tag(t, start, end) }

    // Separators
    rule _() = [Token::Separator]
    rule param_sep() = _? [Token::Comma] _?
    rule stmt_sep() = _? [Token::SemiColon] _?

    // Simpletons: simple composites of atoms
    rule typed_name() -> TypedName =
      name:name() _?
      [Token::Colon] _?
      typ:typ() {
        TypedName {
          name: name,
          typ: typ,
        }
      }

    rule return_spec() -> WithLineInfo<Type> =
      [Token::Arrow] _? typ:typ() { typ }

    // Sequences: things that repeat
    rule params_decl() -> Vec<TypedName> =
      d:(typed_name() ** param_sep()) param_sep()? { d }
    rule fields_decl() -> Vec<TypedName> =
      d:(typed_name() ** param_sep()) param_sep()? { d }
    rule expression_seq() -> Vec<Expression> =
      e:(expression() ** param_sep()) param_sep()? { e }

    // Expression: This beast has a section for itself
    rule expression() -> Expression = precedence! {
      x:(@) _? op:binop(Precedence::Lowest) _? y:@ {
        Expression::BinOp(x.into(), op, y.into())
      }
      --
      x:(@) _? op:binop(Precedence::Low) _? y:@ {
        Expression::BinOp(x.into(), op, y.into())
      }
      --
      x:(@) _? op:binop(Precedence::High) _? y:@ {
        Expression::BinOp(x.into(), op, y.into())
      }
      --
      op:unop() _? x:@ { Expression::UnOp(op, x.into()) }
      --
      atom:atom_boolean() { Expression::AtomBoolean(atom) }
      atom:atom_integer() { Expression::AtomInteger(atom) }
      atom:atom_float() { Expression::AtomFloat(atom) }
      atom:atom_string() { Expression::AtomString(atom) }
      atom:identifier() { Expression::AtomIdentifier(atom) }
      --
      target:call_target() _? [Token::ParenOpen] _? args:expression_seq() _? [Token::ParenClose] {
        Expression::Call(target, args)
      }
      --
      [Token::ParenOpen] _? x:expression() _? [Token::ParenClose] { x }
    }

    // Statements
    rule var_decl() -> Node =
      [Token::Keyword(Keyword::Let)] _
      name:name() _?
      [Token::AssignOp(AssignOp::Identity)] _?
      value:expression() {
        Node::VarDecl(name, None, value)
      }

    rule assignment() -> Node =
      target:identifier() _?
      op:assignop() _?
      value:expression() {
        Node::Assignment(target, op, value)
      }

    rule statement() -> Node =
      e:expression() { Node::Expression(e) } /
      var_decl() /
      assignment()

    rule statement_seq() -> Vec<Node> =
      s:(statement() ** stmt_sep()) stmt_sep()? { s }

    // global declarations
    rule glob_fn_decl() -> Node =
      [Token::Keyword(Keyword::Fn)] _
      name:name() _?
      [Token::ParenOpen] _?
      params:params_decl() _?
      [Token::ParenClose] _?
      return_type:return_spec()? _?
      [Token::BraceOpen] _?
      body:statement_seq() _?
      [Token::BraceClose] {
        Node::FnDecl(name, params, return_type, body)
      }

    rule glob_var_decl() -> Node = d:var_decl() stmt_sep() { d }

    rule glob_mod_decl() -> Node =
      [Token::Keyword(Keyword::Mod)] _
      name:name() stmt_sep() {
        Node::ModDecl(name)
      }

    rule glob_struct_decl() -> Node =
      [Token::Keyword(Keyword::Struct)] _
      name:name() _?
      [Token::BraceOpen] _?
      fields:fields_decl() _?
      [Token::BraceClose] {
        Node::StructDecl(name, fields)
      }

    rule global_decl() -> Node =
      glob_fn_decl() /
      glob_var_decl() /
      glob_mod_decl() /
      glob_struct_decl()

    pub rule global_decl_seq() -> Vec<Node> = global_decl()*
  }
}
