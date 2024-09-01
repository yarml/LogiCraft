mod error;
pub mod module;
pub mod name;
mod symbol;

use super::parser::ast::{Expression, Node};
use crate::{
  pipeline::Tree,
  report::{
    location::WithLineInfo,
    message::{highlight::HighlightType, line::LineType, MessageType},
  },
};
use error::ErrorManager;
use module::ModulePath;
use std::collections::HashMap;
use symbol::{LocalResolver, Resolver};

pub struct Analyzer {
  program: HashMap<ModulePath, Tree>,
}

impl Analyzer {
  pub fn new(program: HashMap<ModulePath, Tree>) -> Self {
    Self { program }
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
    if let Some(first_call) = expr.first_call() {
      let highlight = first_call
        .make_highlight(HighlightType::Focus, Some("Function call here"));
      let WithLineInfo { line, column, .. } = first_call;
      let err_line = errman
        .make_line(line, LineType::Source)
        .with_highlight(highlight);
      errman
        .make_message(
          "Cannot call functions in global context",
          MessageType::Error,
          line,
          column,
        )
        .with_line(err_line)
        .report_and_exit(1);
    }
    let identifiers = expr.all_identifiers();
    for id in &identifiers {
      resolver.resolve(id, errman);
    }
  }
  pub fn resolve_identifiers_in_local_expression(
    &self,
    expr: &Expression,
    resolver: &LocalResolver,
    errman: &mut ErrorManager,
  ) {
    let identifiers = expr.all_identifiers();
    for id in &identifiers {
      resolver.resolve(id, errman);
    }
  }
}
