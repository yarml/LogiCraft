#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
  Mod,
  Use,
  Let,
  Mut,
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
      Keyword::Use => "use",
      Keyword::Let => "let",
      Keyword::Mut => "mut",
      Keyword::Fn => "fn",
      Keyword::If => "if",
      Keyword::Else => "else",
      Keyword::Ret => "return",
      Keyword::Struct => "struct",
    }
  }
}
