use super::{error::ErrorManager, module::ModulePath, symbol::Resolver};
use crate::{
  grammar::parser::ast::Node, pipeline::Tree, report::message::Message,
};

pub struct NameChecker;

impl NameChecker {
  pub fn check(&self, program: &Tree, module: &ModulePath) {
    let mut errman = ErrorManager::new(&program.path, &program.source);
    let mut resolver = Resolver::new(module);

    // First pass find all global names
    for node in &program.nodes {
      match node {
        Node::UseDecl(id) => resolver.use_name(id, &mut errman),
        Node::FnDecl { name, .. } => {
          resolver.declare_name(name.clone(), &mut errman)
        }
        Node::VarDecl { typ, .. } => {
          resolver.declare_name(typ.name.clone(), &mut errman)
        }
        _ => Message::compiler_bug("Found non root node at root")
          .report_and_exit(1),
      }
    }
  }
}
