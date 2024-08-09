use crate::semantics::stage1::validator::StaticType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Builtin {
  Fn(BuiltinFn),
  Type(BuiltinType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltinFn {
  PrintLn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltinType {
  Void,
  Bool,
  Int,
  Float,
  Char,
  String,
}

impl BuiltinFn {
  pub fn static_return_type(&self) -> StaticType {
    match self {
      BuiltinFn::PrintLn => StaticType::Void,
    }
  }
}
