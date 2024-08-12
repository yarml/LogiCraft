use super::ast::{ExpressionNodeData, NodeData, RawExpressionNode, RawNode};
use crate::grammar::{
  builtins::Builtin,
  identifier::{Identifier, Name, Type, TypedNameWithRawLineInfo},
  keywords::Keyword,
  lexer::token::Token,
  operators::{AssignOp, Precedence},
};
use crate::report::location::WithRawLineInfo;
use peg::parser;

parser! {
  pub grammar parser<'a>() for [&'a Token] {
    rule name() -> WithRawLineInfo<Name> =
      start:position!()
      [Token::Identifier(name) if name.is_singular()]
      end:position!() {
        WithRawLineInfo {
          value: name.parts[0].clone(),
          start: start,
          len: end - start
        }
    };

    rule identifier() -> WithRawLineInfo<Identifier> =
      start:position!()
      [Token::Identifier(id)] end:position!() {
        WithRawLineInfo {
          value: id.clone(),
          start: start,
          len: end - start
        }
      };

    rule param_decl() -> TypedNameWithRawLineInfo =
      name:name() [Token::Separator]? [Token::Colon] [Token::Separator]?
      tp_start: position!()
      tp:(
        [Token::Identifier(t)] {
          Type::Declared(t.clone())
        } /
        [Token::Builtin(Builtin::Type(t))] {
          Type::Builtin(*t)
        }
      )
      tp_end:position!() {
        TypedNameWithRawLineInfo(
          name,
          WithRawLineInfo {
            value: tp,
            start: tp_start,
            len: tp_end - tp_start
          }
        )
      }
    rule param_sep() = [Token::Separator]? [Token::Comma] [Token::Separator]?
    rule params_decl() -> Vec<TypedNameWithRawLineInfo> =
      p:(
        param_decl() ** param_sep()
      ) param_sep()? { p }

    rule params() -> Vec<RawExpressionNode> =
      p:(
        expression() ** param_sep()
      ) param_sep()? { p }

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

    rule assignop() -> WithRawLineInfo<AssignOp> =
      start:position!()
      [Token::AssignOp(op)] end:position!() {
        WithRawLineInfo {
          value: *op,
          start: start,
          len: end - start
        }
      };

    rule assignment() -> RawNode =
      start:position!()
      id:identifier() [Token::Separator]?
      op:assignop() [Token::Separator]?
      e:expression()
      end:position!() {
        RawNode {
          data: NodeData::Assignment(id, op, e),
          start: start,
          len: end - start
        }
      };
    rule var_decl() -> RawNode =
      start:position!()
        [Token::Keyword(Keyword::Let)] [Token::Separator]
        name:name() [Token::Separator]?
        type_info:([Token::Colon] [Token::Separator]?
          tp_start:position!()
          tn:(
            [Token::Identifier(t)] {
              Type::Declared(t.clone())
            } /
            [Token::Builtin(Builtin::Type(t))] {
              Type::Builtin(*t)
            }
          )?
          tp_end:position!() {
            (tn, tp_start, tp_end)
          }
        ) [Token::Separator]?
        [Token::AssignOp(AssignOp::Identity)] [Token::Separator]?
        e:expression()
        end:position!() {
          let (tn, tp_start, tp_end) = type_info;
          RawNode {
            data: NodeData::VarDecl(
              tn.map(|t| TypedNameWithRawLineInfo(
                  name,
                  WithRawLineInfo {
                    value: t,
                    start: tp_start,
                    len: tp_end - tp_start
                  }
                )),
              e),
            start: start,
            len: end - start
          }
        };

    rule statement() -> RawNode =
      start:position!() e:expression() end:position!() {
        RawNode {
          data: NodeData::Expression(e),
          start,
          len: end - start,
        }
      } /
      assignment() /
      var_decl()

    rule statement_sep() =
      [Token::Separator]? [Token::SemiColon] [Token::Separator]?

    rule statements() -> Vec<RawNode> =
      s:(
        statement() ** statement_sep()
      ) statement_sep()? { s }

    rule global_fn_decl() -> RawNode =
      start:position!()
      [Token::Keyword(Keyword::Fn)] [Token::Separator]
      name:name() [Token::Separator]?
      [Token::ParenOpen] [Token::Separator]?
      params:params_decl() [Token::Separator]?
      [Token::ParenClose] [Token::Separator]?
      ret_type:(
        [Token::Arrow] [Token::Separator]? start:position!() t:(
          [Token::Identifier(t)] { Type::Declared(t.clone()) } /
          [Token::Builtin(Builtin::Type(t))] { Type::Builtin(*t) }
        ) end:position!() [Token::Separator]? {
          WithRawLineInfo {
            value: t,
            start,
            len: end-start
          }
        }
      )?
      [Token::BraceOpen] [Token::Separator]?
      stmts:statements() [Token::Separator]?
      [Token::BraceClose]
      end:position!() {
        RawNode {
          data: NodeData::FnDecl(
            name,
            params,
            ret_type,
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

    pub rule decls() -> Vec<RawNode> =
      decl() ** ([Token::Separator]?)
  }
}
