pub mod ast;
pub mod module;
pub mod program;
pub mod resolver;

use crate::pipeline::Tree;
use module::ModulePath;
use program::Program;
use resolver::NameResolver;
use std::collections::{HashMap, HashSet};

use super::{identifier::FullIdentifier, parser::ast::Node};

// Made up name to mean "semantic analysis doer & minimizer"
pub struct Semifier;

impl Semifier {
  pub fn semify(&self, modules: HashMap<ModulePath, Tree>) -> Program {
    let mut program = Program::new();
    let mut dependencies = HashSet::new();

    // First pass load all independent functions
    for (module, tree) in modules {
      let mut resolver = NameResolver::new(module.clone());
      // First pass in each function register names in the resolver
      for node in &tree.nodes {
        // TODO:
        // match node {
        //   Node::FnDecl { name, .. } => resolver.use_name(relpath)
        // }
      }
      for node in &tree.nodes {
        match node {
          Node::FnDecl {
            attributes, name, ..
          } => {
            for attr in attributes {
              if attr.value.independent() {
                let full_path =
                  FullIdentifier::compose_global(&module, &name.value);
                let deps =
                  program.load_function(full_path, &mut resolver, node.clone());
                for dep in deps.into_iter().filter(|dep| dep.global()) {
                  dependencies.insert(dep);
                }
              }
            }
          }
          _ => {}
        }
      }
    }

    // Second pass load all dependencies
    for dep in dependencies {

    }

    program
  }
}
