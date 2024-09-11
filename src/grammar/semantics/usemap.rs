use std::{collections::HashMap, fmt::Display};

use crate::grammar::identifier::{GlobalIdentifier, Name};

#[derive(Debug, Clone)]
pub struct UseMap {
  pub usemap: HashMap<Name, GlobalIdentifier>,
}

impl UseMap {
  pub fn lookup(&self, name: &str) -> Option<&GlobalIdentifier> {
    self.usemap.get(name)
  }
}

impl From<HashMap<Name, GlobalIdentifier>> for UseMap {
  fn from(usemap: HashMap<Name, GlobalIdentifier>) -> Self {
    Self { usemap }
  }
}

impl Display for UseMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for (name, id) in &self.usemap {
      writeln!(f, "use {id} as {name}")?;
    }
    Ok(())
  }
}
