use crate::{
  grammar::{
    lexer::Lexer,
    parser::{ast::Node, Parser},
    semantics::{module::ModulePath, Semifier},
  },
  report::message::{Message, MessageType},
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

#[derive(Debug, Clone)]
pub struct Tree {
  pub nodes: Vec<Node>,
  pub path: PathBuf,
  pub source: String,
}

impl Pipeline {
  pub fn new(root: &PathBuf) -> Self {
    Pipeline { root: root.clone() }
  }

  fn load(&self) -> HashMap<ModulePath, Tree> {
    let loader = ModuleLoader;

    loader.load(&self.root, ModulePath::main())
  }

  pub fn run(&self) {
    let program = self.load();
    let declmap = Semifier::declmap(&program);
    println!("{declmap}");
  }
}

impl ModuleLoader {
  fn load(
    &self,
    root: &PathBuf,
    module: ModulePath,
  ) -> HashMap<ModulePath, Tree> {
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
        Message::new(
          &format!("Could not find module `{}`", next.to_string()),
          MessageType::Error,
        )
        .report_and_exit(1)
      }
      if valid_paths.len() > 1 {
        let paths = valid_paths
          .iter()
          .map(|path| format!("`{}`", path.to_string_lossy().to_string()))
          .collect::<Vec<_>>()
          .join(", ");
        Message::new(
          &format!("Ambiguous module `{}`", next.to_string()),
          MessageType::Error,
        )
        .with_note(&format!(
          "Module `{}` could be any of {}",
          next.to_string(),
          paths
        ))
        .report_and_exit(1)
      }
      let path = valid_paths[0].clone();
      let source = fs::read_to_string(&path).unwrap_or_else(|err| {
        Message::input_error(err, &path).report_and_exit(1)
      });
      let lexer = Lexer;
      let parser = Parser;
      let tokens = match lexer.lex(&source) {
        Ok(tokens) => tokens,
        Err(e) => e.get_report(&path, &source).report_and_exit(1),
      };

      let nodes = match parser.parse(&tokens) {
        Ok(nodes) => nodes,
        Err(e) => e.get_report(&path, &source).report_and_exit(1),
      };

      for node in &nodes {
        match node {
          Node::ModDecl(name) => {
            let path = next.join(name.value.clone());
            schedule.insert(path);
          }
          _ => {}
        }
      }
      loaded.insert(
        next.clone(),
        Tree {
          nodes: nodes.clone(),
          path,
          source,
        },
      );
    }
    loaded
  }
}
