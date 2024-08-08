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

pub struct TypedName(Name, Type);

