use std::fmt::Display;

use crate::report::{location::WithLineInfo, message::Message};

use super::{
  builtins::{BuiltinFn, BuiltinType},
  semantics::module::ModulePath,
};

pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalIdentifier {
  pub root: bool,
  pub parts: Vec<WithLineInfo<Name>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalIdentifier {
  pub module: ModulePath,
  pub name: WithLineInfo<Name>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Builtin(BuiltinType),
  Declared(LocalIdentifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallTarget {
  Builtin(BuiltinFn),
  Declared(LocalIdentifier),
}

impl LocalIdentifier {
  pub fn is_singular(&self) -> bool {
    !self.root && self.parts.len() == 1
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

impl Display for LocalIdentifier {
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
    write!(f, "{}::{}", self.module, self.name.value)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::Builtin(btyp) => write!(f, "{btyp}"),
      Type::Declared(id) => write!(f, "{id}"),
    }
  }
}
