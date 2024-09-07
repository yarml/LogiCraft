use super::module::ModulePath;
use crate::grammar::identifier::{GlobalIdentifier, Name, Type};
use std::collections::HashMap;

// ProgDeclMap: Keeps track of all global declarations within all modules.
// This includes the module, name, and type.
pub struct ProgDeclMap {
  decls: HashMap<GlobalIdentifier, GlobalDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDecl {
  pub id: GlobalIdentifier,
  pub ty: GlobalDeclType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalDeclType {
  Variable { typ: Type, mutable: bool },
  Function { args: Vec<Type>, ret: Type },
}

impl ProgDeclMap {
  pub fn new() -> Self {
    ProgDeclMap {
      decls: HashMap::new(),
    }
  }

  pub fn add_decl(&mut self, id: GlobalIdentifier, ty: GlobalDeclType) {
    self.decls.insert(id.clone(), GlobalDecl { id, ty });
  }

  pub fn add_fn(
    &mut self,
    module: ModulePath,
    name: Name,
    args: Vec<Type>,
    ret: Type,
  ) {
    let id = GlobalIdentifier { module, name };
    self.add_decl(id, GlobalDeclType::Function { args, ret });
  }

  pub fn add_var(
    &mut self,
    module: ModulePath,
    name: Name,
    typ: Type,
    mutable: bool,
  ) {
    let id = GlobalIdentifier { module, name };
    self.add_decl(id, GlobalDeclType::Variable { typ, mutable });
  }

  pub fn lookup(&self, id: &GlobalIdentifier) -> Option<&GlobalDecl> {
    self.decls.get(id)
  }
}
