use crate::report::{location::WithLineInfo, message::Message};

use super::{
  builtins::{BuiltinFn, BuiltinType},
  semifier::module::ModulePath,
};

pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
  pub root: bool,
  pub parts: Vec<WithLineInfo<Name>>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum FullIdentifier {
  Local(Name),
  Global(Vec<Name>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Builtin(BuiltinType),
  Declared(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallTarget<I: Clone> {
  Builtin(BuiltinFn),
  Declared(I),
}

impl Identifier {
  pub fn from_name(name: WithLineInfo<Name>) -> Self {
    Identifier {
      root: false,
      parts: vec![name],
    }
  }

  pub fn is_singular(&self) -> bool {
    !self.root && self.parts.len() == 1
  }
  pub fn name(&self) -> Name {
    self.parts.last().unwrap().value.clone()
  }

  pub fn name_line_info(&self) -> WithLineInfo<Name> {
    self.parts.last().unwrap().clone()
  }

  pub fn line_info(&self) -> WithLineInfo<Name> {
    self.parts.first().unwrap().clone()
  }

  pub fn full_path(&self, current_path: ModulePath) -> FullIdentifier {
    if self.root {
      FullIdentifier::Global(
        self.parts.iter().map(|part| part.value.clone()).collect(),
      )
    } else {
      let mut path = current_path.0.clone();
      path.extend(self.parts.iter().map(|part| part.value.clone()));
      FullIdentifier::Global(path)
    }
  }
}

impl FullIdentifier {
  pub fn compose_global(module: &ModulePath, name: &str) -> Self {
    let mut path = module.clone().0;
    path.push(name.to_string());
    FullIdentifier::Global(path)
  }

  pub fn local(&self) -> bool {
    matches!(self, FullIdentifier::Local(_))
  }

  pub fn global(&self) -> bool {
    matches!(self, FullIdentifier::Global(_))
  }

  pub fn module_path(&self) -> ModulePath {
    match self {
      FullIdentifier::Local(_) => {
        Message::compiler_bug("Local identifier has no module path")
          .report_and_exit(1)
      }
      FullIdentifier::Global(path) => {
        ModulePath(path[..path.len() - 1].to_vec())
      }
    }
  }

  pub fn name(&self) -> Name {
    match self {
      FullIdentifier::Local(name) => name.clone(),
      FullIdentifier::Global(path) => path.last().unwrap().clone(),
    }
  }
}
