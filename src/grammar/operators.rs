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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Precedence {
  Lowest,
  Low,
  High,
}

impl Op {
  pub fn can_be_unary(&self) -> bool {
    match self {
      Op::Un(_) => true,
      Op::RawAdd => true,
      Op::RawSub => true,
      _ => false,
    }
  }

  pub fn as_unary(&self) -> UnOp {
    match self {
      Op::Un(op) => *op,
      Op::RawAdd => UnOp::Identity,
      Op::RawSub => UnOp::Negate,
      _ => panic!("Cannot convert binary operator to unary"),
    }
  }

  pub fn can_be_binary(&self) -> bool {
    match self {
      Op::Bin(_) => true,
      Op::RawAdd => true,
      Op::RawSub => true,
      _ => false,
    }
  }

  pub fn as_binary(&self) -> BinOp {
    match self {
      Op::Bin(op) => *op,
      Op::RawAdd => BinOp::Add,
      Op::RawSub => BinOp::Sub,
      _ => panic!("Cannot convert unary operator to binary"),
    }
  }

  pub fn binary_with(&self, precedence: Precedence) -> bool {
    self.can_be_binary() && self.as_binary().precedence() == precedence
  }
}

impl BinOp {
  pub fn precedence(&self) -> Precedence {
    match self {
      BinOp::Add => Precedence::Low,
      BinOp::Sub => Precedence::Low,
      BinOp::Mul => Precedence::High,
      BinOp::Div => Precedence::High,
      BinOp::Mod => Precedence::High,
      BinOp::Equal => Precedence::Lowest,
      BinOp::NotEqual => Precedence::Lowest,
      BinOp::Less => Precedence::Lowest,
      BinOp::LessOrEqual => Precedence::Lowest,
      BinOp::Greater => Precedence::Lowest,
      BinOp::GreaterOrEqual => Precedence::Lowest,
    }
  }
}
