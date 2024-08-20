mod error;
pub mod module;
mod symbol;

use crate::{
  pipeline::Tree,
  report::{
    location::WithLineInfo,
    message::{highlight::HighlightType, line::LineType, Message, MessageType},
  },
};

use super::{
  identifier::{CallTarget, Identifier},
  parser::ast::{Expression, Node},
};
use error::ErrorManager;
use module::ModulePath;
use std::collections::HashMap;
use symbol::{LocalResolver, Resolver};

pub struct Analyzer {
  program: HashMap<ModulePath, Tree>,
  analyzed: HashMap<ModulePath, ()>,
}

impl Analyzer {
  pub fn new(program: HashMap<ModulePath, Tree>) -> Self {
    Self {
      program,
      analyzed: HashMap::new(),
    }
  }

  pub fn analyze(&self, module: &ModulePath) {
    let tree = self.program.get(module).unwrap();
    let mut errman = ErrorManager::new(&tree.path, &tree.source);
    let mut resolver = Resolver::new(module);
    // First pass find all global names
    for node in &tree.nodes {
      match node {
        Node::UseDecl(id) => resolver.use_name(id, &mut errman),
        Node::FnDecl { name, .. } => {
          resolver.declare_name(name.clone(), &mut errman)
        }
        Node::VarDecl { typ, .. } => {
          resolver.declare_name(typ.name.clone(), &mut errman)
        }
        _ => {}
      }
    }
    // Second pass validate identifiers within expressions
    for node in &tree.nodes {
      self.resolve_identifiers_in_node(node, &resolver, &mut errman);
    }
  }

  pub fn resolve_identifiers_in_node(
    &self,
    node: &Node,
    resolver: &Resolver,
    errman: &mut ErrorManager,
  ) {
    match node {
      Node::VarDecl { val, .. } => {
        self.resolve_identifiers_in_global_expression(val, resolver, errman)
      }
      Node::FnDecl { body, .. } => {
        self.resolve_fn_body(&body, resolver, errman)
      }
      _ => {}
    }
  }

  pub fn resolve_fn_body(
    &self,
    body: &[Node],
    resolver: &Resolver,
    errman: &mut ErrorManager,
  ) {
    let mut local_resolver = LocalResolver::new(resolver);
    for node in body {
      match node {
        Node::VarDecl { typ, val } => {
          self.resolve_identifiers_in_local_expression(
            val,
            &local_resolver,
            errman,
          );
          local_resolver.declare_name(typ.name.clone(), errman);
        }
        Node::Assignment { target, val, .. } => {
          self.resolve_identifiers_in_local_expression(
            val,
            &local_resolver,
            errman,
          );
          local_resolver.resolve(target, errman);
        }
        Node::Expression(expr) => self.resolve_identifiers_in_local_expression(
          expr,
          &local_resolver,
          errman,
        ),
        Node::Return(expr) => self.resolve_identifiers_in_local_expression(
          expr,
          &local_resolver,
          errman,
        ),
        _ => {}
      }
    }
  }

  pub fn resolve_identifiers_in_global_expression(
    &self,
    expr: &Expression,
    resolver: &Resolver,
    errman: &mut ErrorManager,
  ) {
    match expr {
      Expression::AtomIdentifier(id) => {
        resolver.resolve(id, errman);
      }
      Expression::BinOp(left, _, right) => {
        self.resolve_identifiers_in_global_expression(left, resolver, errman);
        self.resolve_identifiers_in_global_expression(right, resolver, errman);
      }
      Expression::UnOp(_, expr) => {
        self.resolve_identifiers_in_global_expression(expr, resolver, errman);
      }
      Expression::Call(target, _) => {
        let highlight = target
          .make_highligh(HighlightType::Focus, Some("Function call here"));
        let WithLineInfo { line, column, .. } = target;
        let err_line = errman
          .make_line(*line, LineType::Source)
          .with_highlight(highlight);
        errman
          .make_message(
            "Cannot call functions in global context",
            MessageType::Error,
            *line,
            *column,
          )
          .with_line(err_line)
          .report_and_exit(1);
      }
      _ => {}
    }
  }
  pub fn resolve_identifiers_in_local_expression(
    &self,
    expr: &Expression,
    resolver: &LocalResolver,
    errman: &mut ErrorManager,
  ) {
    match expr {
      Expression::AtomIdentifier(id) => {
        resolver.resolve(id, errman);
      }
      Expression::BinOp(left, _, right) => {
        self.resolve_identifiers_in_local_expression(left, resolver, errman);
        self.resolve_identifiers_in_local_expression(right, resolver, errman);
      }
      Expression::UnOp(_, expr) => {
        self.resolve_identifiers_in_local_expression(expr, resolver, errman);
      }
      Expression::Call(target, params) => {
        if let CallTarget::Declared(id) = &target.value {
          resolver.resolve(id, errman);
        }
        for param in params {
          self.resolve_identifiers_in_local_expression(param, resolver, errman);
        }
      }
      _ => {}
    }
  }
}
