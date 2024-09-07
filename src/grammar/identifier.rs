use crate::report::location::WithLineInfo;

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
  pub name: Name,
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
