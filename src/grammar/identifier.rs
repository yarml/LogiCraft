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
  pub fn name(&self) -> Name {
    self.parts.last().unwrap().value.clone()
  }

  pub fn name_line_info(&self) -> WithLineInfo<Name> {
    self.parts.last().unwrap().clone()
  }

  pub fn line_info(&self) -> WithLineInfo<Name> {
    self.parts.first().unwrap().clone()
  }
}
