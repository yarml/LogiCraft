use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Builtin {
  Fn(BuiltinFn),
  Type(BuiltinType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltinFn {
  PrintLn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
  Void,
  Bool,
  Int,
  Float,
  Char,
  String,
}

impl Display for BuiltinType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        BuiltinType::Void => "void",
        BuiltinType::Bool => "bool",
        BuiltinType::Int => "int",
        BuiltinType::Float => "float",
        BuiltinType::Char => "char",
        BuiltinType::String => "string",
      }
    )
  }
}
