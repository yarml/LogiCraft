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