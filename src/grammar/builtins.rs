#[derive(Debug, Clone, PartialEq)]
pub enum Builtin {
  Fn(BuiltinFn),
  Type(BuiltinType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFn {
  PrintLn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinType {
  Bool,
  Int,
  Float,
  Char,
  String,
}
