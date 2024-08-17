use super::{error::ErrorManager, module::ModulePath};
use crate::{
  grammar::identifier::{Identifier, Name},
  report::{
    location::WithLineInfo,
    message::{
      highlight::{Highlight, HighlightType},
      line::{HighlightedLine, LineType},
      MessageType,
    },
  },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Resolver {
  module: ModulePath,
  aliases: HashMap<Name, Alias>,
  parent: Option<Box<Resolver>>,
}

#[derive(Debug, Clone)]
pub struct Alias {
  pub path: Vec<Name>,
  alias: WithLineInfo<Name>,
}

impl Resolver {
  pub fn new(module: &ModulePath, parent: Option<Resolver>) -> Self {
    Self {
      module: module.clone(),
      aliases: HashMap::new(),
      parent: parent.map(|p| Box::new(p)),
    }
  }

  pub fn pop(self) -> Option<Resolver> {
    self.parent.map(|p| *p)
  }

  pub fn use_name(&mut self, id: &Identifier, errman: &mut ErrorManager) {
    println!("Using: {id:?}");
    let name = id.name();
    let new_alias = Alias::new(id, &self.module);
    if self.aliases.contains_key(&name) {
      let line_info = id.name_line_info();
      let prev_use = &self.aliases[&name];
      errman
        .make_message(
          &format!("Name `{name}` already used"),
          MessageType::Error,
          line_info.line,
          line_info.column,
        )
        .with_line(new_alias.make_line(
          LineType::Source,
          HighlightType::Focus,
          Some("Name already used"),
          errman,
        ))
        .with_line(prev_use.make_line(
          LineType::Source,
          HighlightType::Helper,
          Some("Previously used here"),
          errman,
        ))
        .report_and_exit(1)
    }
    self.aliases.insert(name, new_alias);
  }
}

impl Alias {
  pub fn new(id: &Identifier, module: &ModulePath) -> Self {
    let mut parts = id.parts.iter().map(|part| part.value.clone()).collect();
    if id.root {
      Self {
        alias: id.name_line_info(),
        path: parts,
      }
    } else {
      let mut full_path = module.0.clone();
      full_path.append(&mut parts);
      Self {
        alias: id.name_line_info(),
        path: full_path,
      }
    }
  }

  pub fn make_line(
    &self,
    ltyp: LineType,
    htyp: HighlightType,
    label: Option<&str>,
    errman: &ErrorManager,
  ) -> HighlightedLine {
    let mut highlight = Highlight::new(self.alias.column, self.alias.len, htyp);
    if let Some(label) = label {
      highlight.add_label(label);
    }
    errman
      .make_line(self.alias.line, ltyp)
      .with_highlight(highlight)
  }
}
