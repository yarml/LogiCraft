use std::path::PathBuf;

use crate::{
  grammar::identifier::{Identifier, Name},
  report::message::Message,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(pub Vec<Name>);

impl ModulePath {
  pub fn main() -> Self {
    ModulePath(vec![])
  }
  pub fn of_root_identifier(id: &Identifier) -> Self {
    if id.root {
      let mut parts: Vec<_> =
        id.parts.iter().map(|part| part.value.clone()).collect();
      parts.pop();
      ModulePath(parts)
    } else {
      Message::compiler_bug(
        "ModulePath::of_root_identifier called with non-root identifier",
      )
      .report_and_exit(1)
    }
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

impl ToString for ModulePath {
  fn to_string(&self) -> String {
    if self.0.len() == 0 {
      "main".to_string()
    } else {
      self
        .0
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>()
        .join("::")
    }
  }
}
