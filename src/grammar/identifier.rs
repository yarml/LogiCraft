use super::builtins::BuiltinType;

pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
  pub root: bool,
  pub parts: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Builtin(BuiltinType),
  Declared(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedName(pub Name, pub Type);

impl Identifier {
  pub fn is_singular(&self) -> bool {
    !self.root && self.parts.len() == 1
  }
}

