use super::{ast::FnDecl, resolver::NameResolver};
use crate::grammar::{identifier::FullIdentifier, parser::ast::Node};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program {
  functions: HashMap<FullIdentifier, FnDecl>,
}

impl Program {
  pub fn new() -> Self {
    Program {
      functions: HashMap::new(),
    }
  }

  // Adds a function to the program and returns its dependencies
  pub fn load_function(
    &mut self,
    path: FullIdentifier,
    resolver: &mut NameResolver,
    fnnode: Node,
  ) -> Vec<FullIdentifier> {
    let function = FnDecl::from_function_node(fnnode, resolver);
    let deps = function.dependencies.clone();
    self.functions.insert(path, function);
    deps
  }
}
