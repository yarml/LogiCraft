use std::fmt::Display;

use crate::report::{location::WithLineInfo, message::Message};

use super::{
  builtins::{BuiltinFn, BuiltinType},
  parser::ast::TypedNameLI,
  semantics::{decl::GlobalDecl, module::ModulePath},
};

pub type Name = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnscopedIdentifier {
  pub root: bool,
  pub parts: Vec<WithLineInfo<Name>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalIdentifier {
  pub module: ModulePath,
  pub name: WithLineInfo<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopedIdentifier {
  Local(TypedNameLI<GlobalIdentifier>),
  Global(GlobalDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<I: Clone> {
  Builtin(BuiltinType),
  Declared(I),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallTarget<I: Clone> {
  Builtin(BuiltinFn),
  Declared(I),
}

impl UnscopedIdentifier {
  pub fn is_singular(&self) -> bool {
    !self.root && self.parts.len() == 1
  }

  pub fn resolve(&self, reference: &ModulePath) -> GlobalIdentifier {
    if self.root {
      self.as_root()
    } else {
      self.from_ref(reference)
    }
  }

  pub fn as_root(&self) -> GlobalIdentifier {
    let module = ModulePath(
      self.parts[..self.parts.len() - 1]
        .iter()
        .map(|winfo| winfo.value.clone())
        .collect::<Vec<_>>(),
    );
    let name = self.parts.last().unwrap().clone();
    GlobalIdentifier { module, name }
  }
  pub fn from_ref(&self, reference: &ModulePath) -> GlobalIdentifier {
    let mut module = reference.clone();
    module.0.extend_from_slice(
      &self.parts[..self.parts.len() - 1]
        .iter()
        .map(|winfo| winfo.value.clone())
        .collect::<Vec<_>>(),
    );
    let name = self.parts.last().unwrap().clone();
    GlobalIdentifier { module, name }
  }

  pub fn name(&self) -> Option<WithLineInfo<Name>> {
    self.parts.last().cloned()
  }
}

impl From<&str> for UnscopedIdentifier {
  fn from(value: &str) -> Self {
    let parts = value.split("::").collect::<Vec<_>>();
    let root = parts[0] == "";
    let parts = if root { &parts[1..] } else { &parts[..] };
    let parts = parts
      .iter()
      .map(|part| WithLineInfo::debug(part.to_string()))
      .collect();
    UnscopedIdentifier { root, parts }
  }
}

impl GlobalIdentifier {
  pub fn push_names(&mut self, names: &[Name]) {
    self.module.join(self.name.value.clone());
    self.module.0.extend_from_slice(&names[..names.len() - 1]);
    self.name = WithLineInfo::debug(names.last().unwrap().clone());
  }
}

impl From<&str> for GlobalIdentifier {
  fn from(value: &str) -> Self {
    let parts = value.split("::").collect::<Vec<_>>();
    if parts.len() == 1 {
      Message::compiler_bug("A global identifier needs at least 2 parts")
        .report_and_exit(1);
    }
    if parts[0] == "lib" {
      if parts.len() == 2 {
        return GlobalIdentifier {
          module: ModulePath::main(),
          name: WithLineInfo::debug(parts[1].to_string()),
        };
      } else {
        Message::compiler_bug("Main module cannot have more than 2 parts")
          .report_and_exit(1);
      }
    }

    let module = ModulePath(
      parts[..parts.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    );
    let name = WithLineInfo::debug(parts[parts.len() - 1].to_string());
    GlobalIdentifier { module, name }
  }
}

impl Display for UnscopedIdentifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let prefix = if self.root { "::" } else { "" };
    let name = self
      .parts
      .iter()
      .map(|winfo| winfo.value.clone())
      .collect::<Vec<_>>()
      .join("::");
    write!(f, "{prefix}{name}")
  }
}

impl Display for GlobalIdentifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "::{}::{}", self.module, self.name.value)
  }
}

impl<I: Clone + Display> Display for Type<I> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Builtin(btyp) => write!(f, "{btyp}"),
      Type::Declared(id) => write!(f, "{id}"),
    }
  }
}
