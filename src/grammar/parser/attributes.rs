#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
  Export,
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
