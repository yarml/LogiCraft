use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PackMeta {
  description: String,
  format: usize,
}

impl PackMeta {
  pub fn new(description: &str) -> Self {
    Self {
      description: description.to_string(),
      format: 48,
    }
  }
}
