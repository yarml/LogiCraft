mod error;
pub mod module;
mod symbol;

use crate::{pipeline::Tree, report::location::WithLineInfo};

use super::{identifier::Identifier, parser::ast::Node};
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
    let mut resolver = Resolver::new(module);
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
    let resolved = resolver.resolve(
      &Identifier {
        root: false,
        parts: vec![WithLineInfo {
          value: String::from("getnum"),
          line: 6,
          column: 11,
          len: 6,
        }],
      },
      &mut errman,
    );
    println!("{:?}", resolved);
  }
}
