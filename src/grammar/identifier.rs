use crate::report::location::WithLineInfo;

use super::builtins::{BuiltinFn, BuiltinType};

pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
  pub root: bool,
  pub parts: Vec<WithLineInfo<Name>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Builtin(BuiltinType),
  Declared(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallTarget {
  Builtin(BuiltinFn),
  Declared(Identifier),
}

impl Identifier {
  pub fn is_singular(&self) -> bool {
    !self.root && self.parts.len() == 1
  }
}
