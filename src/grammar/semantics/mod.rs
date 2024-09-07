pub mod decl;
pub mod module;
pub mod usemap;

use super::{identifier::GlobalIdentifier, parser::ast::Node};
use crate::{
  pipeline::Tree,
  report::message::{
    highlight::{self, Highlight, HighlightType},
    line::{HighlightedLine, LineType},
    Message, MessageMeta, MessageType,
  },
};
use decl::ProgDeclMap;
use module::ModulePath;
use std::collections::HashMap;
use usemap::UseMap;

pub struct Semifier;

impl Semifier {
  pub fn declmap(program: &HashMap<ModulePath, Tree>) -> ProgDeclMap {
    let mut decl_map = ProgDeclMap::new();
    for (module_path, tree) in program {
      decl_map.add_module(module_path, tree);
    }
    decl_map
  }
  pub fn usemap(
    module_path: &ModulePath,
    declmap: &ProgDeclMap,
    tree: &Tree,
  ) -> UseMap {
    let usemap = tree
      .nodes
      .iter()
      .filter_map(|node| match node {
        Node::UseDecl(id) => {
          let resolved = id.resolve(module_path);
          Some((resolved.name.value.clone(), resolved))
        }
        _ => None,
      })
      .collect::<HashMap<_, _>>();

    // Verify all values within usemap exist in declmap
    for id in usemap.values() {
      if declmap.lookup(id).is_none() {
        let highlight =
          Highlight::new(id.name.column, id.name.len, HighlightType::Focus);
        let line = HighlightedLine::from_src(&tree.source, id.name.line)
          .with_highlight(highlight);
        Message::new(
          &format!("Use of undeclared identifier {}", id.name.value),
          MessageType::Error,
        )
        .with_meta(MessageMeta::FileLocation(
          tree.path.clone(),
          id.name.line,
          id.name.column,
        ))
        .with_line(line)
        .report_and_exit(1);
      }
    }

    UseMap::from(usemap)
  }
}
