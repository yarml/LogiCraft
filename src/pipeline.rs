use crate::{
  grammar::{
    lexer::Lexer,
    parser::{
      ast::{Node, NodeData},
      Parser,
    },
  },
  semantics::module::ModulePath,
};
use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
};

#[derive(Debug, Clone)]
struct ModuleLoader;

pub struct Pipeline {
  root: PathBuf,
}

impl Pipeline {
  pub fn new(root: &PathBuf) -> Self {
    Pipeline { root: root.clone() }
  }

  fn load(&self) -> HashMap<ModulePath, Vec<Node>> {
    let loader = ModuleLoader;

    loader.load(&self.root, ModulePath::main())
  }

  pub fn run(&self) {
    self.load();
  }
}

impl ModuleLoader {
  fn load(
    &self,
    root: &PathBuf,
    module: ModulePath,
  ) -> HashMap<ModulePath, Vec<Node>> {
    let mut schedule = HashSet::from([module]);
    let mut loaded = HashMap::new();

    while !schedule.is_empty() {
      let next = schedule.iter().next().unwrap().clone();
      schedule.remove(&next);

      if loaded.contains_key(&next) {
        continue;
      }
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
      let lexer = Lexer;
      let parser = Parser;
      let tokens = match lexer.lex(&source) {
        Ok(tokens) => tokens,
        Err(e) => e.report_and_exit(&path, &source),
      };

      let nodes = match parser.parse(&tokens) {
        Ok(nodes) => nodes,
        Err(e) => e.report_and_exit(&path, &source),
      };

      for node in &nodes {
        println!("{:?}", node);
        let node = node.data.clone();
        match node {
          NodeData::ModDecl(name) => {
            let path = next.join(name.value.clone());
            schedule.insert(path);
          }
          _ => {}
        }
      }
      loaded.insert(next.clone(), nodes.clone());
    }
    loaded
  }
}
