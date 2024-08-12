#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
  Mod,
  Let,
  Fn,
  If,
  Else,
  Ret,
  Struct,
}

impl Keyword {
  pub fn error_symbol(&self) -> &'static str {
    match self {
      Keyword::Mod => "mod",
      Keyword::Let => "let",
      Keyword::Fn => "fn",
      Keyword::If => "if",
      Keyword::Else => "else",
      Keyword::Ret => "ret",
      Keyword::Struct => "struct",
    }
  }
}
