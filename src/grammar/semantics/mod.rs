use std::collections::HashMap;

use decl::ProgDeclMap;
use module::ModulePath;

use crate::pipeline::Tree;

pub mod decl;
pub mod module;

pub fn semify(program: &HashMap<ModulePath, Tree>) {
  let mut decl_map = ProgDeclMap::new();
  for (module_path, tree) in program {
    decl_map.add_module(module_path, tree);
  }

  println!("{}", decl_map);
}
