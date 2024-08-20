use super::{error::ErrorManager, module::ModulePath};
use crate::{
  grammar::identifier::{Identifier, Name},
  report::{
    location::WithLineInfo,
    message::{
      highlight::{Highlight, HighlightType},
      line::{HighlightedLine, LineType},
      Message, MessageType,
    },
  },
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Resolver {
  module: ModulePath,
  aliases: HashMap<Name, Alias>,
}

#[derive(Debug, Clone)]
pub struct LocalResolver<'a> {
  names: HashMap<Name, WithLineInfo<Name>>,
  parent: &'a Resolver,
}

#[derive(Debug, Clone)]
pub struct Alias {
  pub path: Vec<Name>,
  alias: WithLineInfo<Name>,
}

#[derive(Debug, Clone)]
pub enum ResolvedName {
  Local(Name),
  Global(Vec<Name>),
}

impl Resolver {
  pub fn new(module: &ModulePath) -> Self {
    Self {
      module: module.clone(),
      aliases: HashMap::new(),
    }
  }

  pub fn use_name(&mut self, id: &Identifier, errman: &mut ErrorManager) {
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

  pub fn declare_name(
    &mut self,
    name: WithLineInfo<Name>,
    errman: &mut ErrorManager,
  ) {
    self.use_name(
      &Identifier {
        root: false,
        parts: vec![name],
      },
      errman,
    )
  }

  pub fn resolve(
    &self,
    id: &Identifier,
    errman: &mut ErrorManager,
  ) -> Vec<Name> {
    if id.root {
      return id.parts.iter().map(|part| part.value.clone()).collect();
    }

    let first = id.parts.first().unwrap();
    let alias = self.aliases.get(&first.value);

    if alias.is_none() {
      let line_info = id.line_info();
      let line = errman
        .make_line(line_info.line, LineType::Source)
        .with_highlight(
          Highlight::new(line_info.column, line_info.len, HighlightType::Focus)
            .with_label("This symbol"),
        );
      errman
        .make_message(
          &format!("Could not resolve `{}`", first.value),
          MessageType::Error,
          line_info.line,
          line_info.column,
        )
        .with_line(line)
        .report_and_exit(1)
    }

    let alias = alias.unwrap();
    let mut path = alias.path.clone();
    path.append(
      &mut id
        .parts
        .iter()
        .skip(1)
        .map(|part| part.value.clone())
        .collect(),
    );
    path
  }
}

impl<'a> LocalResolver<'a> {
  pub fn new(parent: &'a Resolver) -> Self {
    Self {
      names: HashMap::new(),
      parent,
    }
  }

  pub fn declare_name(
    &mut self,
    name: WithLineInfo<Name>,
    errman: &mut ErrorManager,
  ) {
    if self.names.contains_key(&name.value) {
      let prev_name = &self.names[&name.value];
      let current_highlight =
        name.make_highligh(HighlightType::Focus, Some("Name already declared"));
      let prev_highlight = prev_name
        .make_highligh(HighlightType::Helper, Some("Previously declared here"));
      let current_line = errman
        .make_line(name.line, LineType::Source)
        .with_highlight(current_highlight);
      let prev_line = errman
        .make_line(prev_name.line, LineType::Source)
        .with_highlight(prev_highlight);
      errman
        .make_message(
          "Name already used",
          MessageType::Error,
          name.line,
          name.column,
        )
        .with_line(current_line)
        .with_line(prev_line)
        .report_and_exit(1)
    }
    self.names.insert(name.value.clone(), name);
  }

  pub fn resolve(
    &self,
    id: &Identifier,
    errman: &mut ErrorManager,
  ) -> ResolvedName {
    if id.is_singular() {
      let name = id.parts.first().unwrap();
      if self.names.contains_key(&name.value) {
        return ResolvedName::Local(name.value.clone());
      }
    }

    ResolvedName::Global(self.parent.resolve(id, errman))
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
