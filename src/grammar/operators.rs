#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
  Not,
  Negate,
  Identity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Equal,
  NotEqual,
  Less,
  LessOrEqual,
  Greater,
  GreaterOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
  Un(UnOp),
  Bin(BinOp),
  // Operators that the lexer cannot know whether they are in unary or binary form
  RawAdd,
  RawSub,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOp {
  Identity,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
}
