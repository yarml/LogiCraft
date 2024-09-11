use crate::{
  grammar::{
    identifier::{GlobalIdentifier, Name, ScopedIdentifier, Type},
    parser::ast::{Expression, TypedName},
  },
  report::location::WithLineInfo,
};
use std::collections::HashMap;

use super::module::ModulePath;

#[derive(Debug, Clone)]
pub struct SemifiedProgram {
  functions: HashMap<GlobalIdentifier, Function>,
  vars: HashMap<GlobalIdentifier, GlobalVar>,
}

#[derive(Debug, Clone)]
pub struct Function {
  attributes: Vec<Name>,
  params: Vec<TypedName<GlobalIdentifier>>,
  locals: Vec<TypedName<GlobalIdentifier>>,
  ret_type: Type<GlobalIdentifier>,
  body: Vec<FunctionNode>,
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
  init: Expression<()>, // I don't wanna deal with using identifiers within global expressions, leaving that for later.
  typ: Type<GlobalIdentifier>,
  mutable: bool,
}

#[derive(Debug, Clone)]
pub enum FunctionNode {
  SideEffect(Expression<ScopedIdentifier>),
  Assignment {
    target: ScopedIdentifier,
    val: Expression<ScopedIdentifier>,
  },
  Return(),
}

impl SemifiedProgram {
  pub fn new() -> Self {
    Self {
      functions: HashMap::new(),
      vars: HashMap::new(),
    }
  }

  pub fn decl_var(
    &mut self,
    module_path: &ModulePath,
    name: WithLineInfo<Name>,
    typ: Type<GlobalIdentifier>,
    init: Expression<()>,
    mutable: bool,
  ) {
    let id = GlobalIdentifier {
      module: module_path.clone(),
      name,
    };
    self.vars.insert(id, GlobalVar { init, typ, mutable });
  }

  pub fn decl_fn() {
    
  }
}
