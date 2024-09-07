#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
  Export,
}

impl Attribute {
  pub fn independent(&self) -> bool {
    match self {
      Self::Export => true,
    }
  }
}

impl TryFrom<&str> for Attribute {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "export" => Ok(Self::Export),
      _ => Err(()),
    }
  }
}
