use crate::{
  grammar::identifier::{FullIdentifier, Identifier, Name},
  report::location::WithLineInfo,
};
use std::collections::HashMap;

use super::module::ModulePath;

#[derive(Debug, Clone)]
pub struct NameResolver {
  current_path: ModulePath,
  aliases: HashMap<Name, ResolvedName>,
  scopes: Vec<Vec<(Name, bool)>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedName {
  pub id: FullIdentifier,
  pub mutable: bool,
}

impl NameResolver {
  pub fn new(self_path: ModulePath) -> Self {
    NameResolver {
      current_path: self_path,
      aliases: HashMap::new(),
      scopes: vec![],
    }
  }

  pub fn use_name(&mut self, relpath: Identifier, mutable: bool) {
    let full_path = relpath.full_path(self.current_path.clone());
    let name = relpath.name();
    self.aliases.insert(
      name,
      ResolvedName {
        id: full_path,
        mutable,
      },
    );
  }
  pub fn decl_global(&mut self, name: WithLineInfo<Name>, mutable: bool) {
    self.use_name(Identifier::from_name(name), mutable)
  }
  pub fn decl_local(&mut self, name: Name, mutable: bool) {
    self.scopes.last_mut().unwrap().push((name, mutable));
  }

  pub fn push_scope(&mut self) {
    self.scopes.push(Vec::new());
  }
  pub fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  pub fn get_local(&self, name: &Name) -> Option<ResolvedName> {
    self
      .scopes
      .iter()
      .rev()
      .filter_map(|scope| {
        scope.iter().find(|(local_name, _)| local_name == name)
      })
      .next()
      .map(|(name, mutable)| ResolvedName {
        id: FullIdentifier::Local(name.clone()),
        mutable: *mutable,
      })
  }

  pub fn resolve(&self, id: &Identifier) -> ResolvedName {
    if id.root {
      let mut path = self.current_path.0.clone();
      path.extend(id.parts.iter().map(|part| part.value.clone()));
      return ResolvedName {
        id: FullIdentifier::Global(path),
        mutable: false,
      };     
    }
    if id.is_singular() {
      if let Some(alias) = self.get_local(&id.name()) {
        return alias;
      }
    }


    FullIdentifier::Global(path)
  }
}
