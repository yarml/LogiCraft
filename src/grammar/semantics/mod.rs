mod error;
pub mod module;
mod symbol;

use crate::pipeline::Tree;

use super::parser::ast::Node;
use error::ErrorManager;
use module::ModulePath;
use std::collections::HashMap;
use symbol::Resolver;

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
    let mut resolver = Resolver::new(module, None);
    for node in &tree.nodes {
      match node {
        Node::UseDecl(id) => resolver.use_name(id, &mut errman),
        _ => {}
      }
    }
  }
}
