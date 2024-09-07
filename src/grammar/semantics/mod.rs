pub mod decl;
pub mod module;

use crate::pipeline::Tree;
use decl::ProgDeclMap;
use module::ModulePath;
use std::collections::HashMap;

pub struct Semifier;

impl Semifier {
  pub fn declmap(program: &HashMap<ModulePath, Tree>) -> ProgDeclMap {
    let mut decl_map = ProgDeclMap::new();
    for (module_path, tree) in program {
      decl_map.add_module(module_path, tree);
    }
    decl_map
  }
}
