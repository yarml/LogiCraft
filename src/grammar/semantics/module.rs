use std::{fmt::Display, path::PathBuf};

use crate::grammar::identifier::{GlobalIdentifier, LocalIdentifier, Name};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(pub Vec<Name>);

impl ModulePath {
  pub fn main() -> Self {
    ModulePath(vec![])
  }

  pub fn paths(&self, root: PathBuf) -> Vec<PathBuf> {
    if self.0.len() == 0 {
      vec![root.join("lib.lc")]
    } else {
      let names_except_last = self.0[..self.0.len() - 1].iter();
      let last_name = self.0.last().unwrap();
      let root = names_except_last.fold(root, |acc, name| acc.join(name));
      vec![
        root.join(&format!("{last_name}.lc")),
        root.join(last_name).join("mod.lc"),
      ]
    }
  }

  pub fn join(&self, name: Name) -> ModulePath {
    let mut new_path = self.0.clone();
    new_path.push(name);
    ModulePath(new_path)
  }
}

impl From<&str> for ModulePath {
  fn from(value: &str) -> Self {
    let parts = value.split("::").collect::<Vec<_>>();
    let parts = parts
      .iter()
      .map(|part| part.to_string())
      .collect::<Vec<_>>();
    ModulePath(parts)
  }
}

impl Display for ModulePath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name = if self.0.len() == 0 {
      "lib".to_string()
    } else {
      self
        .0
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
        .join("::")
    };
    write!(f, "{name}")
  }
}
