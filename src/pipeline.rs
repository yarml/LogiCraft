use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
};

use crate::{
  grammar::{
    lexer::lexer,
    parser::{parser, Node},
  },
  semantics::{module::ModulePath, stage1::{self, optimizer::ast_optimize, validator::validate}},
};

#[derive(Debug, Clone)]
struct LoadContext {
  // Modules scheduled to load before exiting the load stage
  schedule: HashSet<ModulePath>,
  loaded: HashMap<ModulePath, Vec<Node>>,
}

pub struct Pipeline {
  root: PathBuf,
}

impl Pipeline {
  pub fn new(root: &PathBuf) -> Self {
    Pipeline { root: root.clone() }
  }

  fn load(&self) -> HashMap<ModulePath, Vec<Node>> {
    let mut context = LoadContext {
      schedule: HashSet::new(),
      loaded: HashMap::new(),
    };

    context.load(&self.root, ModulePath::main());

    context.loaded
  }

  fn stage1_optimize(nodes: &mut Vec<Node>) {
    ast_optimize(nodes);
  }
  fn stage1_validate(nodes: &[Node]) {
    stage1::validator::validate(nodes);
  }

  pub fn run(&self) {
    let mut modules = self.load();

    // Stage 1 Optimization & Validation
    for nodes in modules.values_mut() {
      Pipeline::stage1_optimize(nodes);
    }
  }
}

impl LoadContext {
  fn load(&mut self, root: &PathBuf, module: ModulePath) {
    self.schedule.insert(module);

    while !self.schedule.is_empty() {
      let next = self.schedule.iter().next().unwrap().clone();
      self.schedule.remove(&next);

      if self.loaded.contains_key(&next) {
        continue;
      }

      println!("Loading module: {}", next.to_string());

      let all_paths = next.paths(root.clone());
      let valid_paths: Vec<_> = all_paths
        .into_iter()
        .filter(|path| match fs::metadata(path) {
          Ok(metadata) => metadata.is_file(),
          Err(_) => false,
        })
        .collect();
      if valid_paths.is_empty() {
        panic!("Could not find module {}", next.to_string());
      }
      if valid_paths.len() > 1 {
        panic!("Ambiguous module {}", next.to_string());
      }
      let path = valid_paths[0].clone();

      let source = fs::read_to_string(&path)
        .expect(&format!("Could not read module: {:?}", path));
      let tokens = lexer::lex(&source)
        .expect(&format!("Could not lex module: {:?}", path));
      let token_ref: Vec<_> = tokens.iter().map(|t| t).collect();
      let nodes = parser::decl(&token_ref[..])
        .expect(&format!("Could not parse module: {:?}", path));

      for node in &nodes {
        println!("{:?}", node);
        match node {
          Node::ModDecl(name) => {
            let path = next.join(name.clone());
            self.schedule.insert(path);
          }
          _ => {}
        }
      }
      self.loaded.insert(next.clone(), nodes.clone());
    }
  }
}
