pub mod ast;
pub mod decl;
pub mod module;
pub mod usemap;

use super::{
  identifier::{GlobalIdentifier, ScopedIdentifier},
  parser::ast::{Node, TypedNameLI},
};
use crate::{
  pipeline::Tree,
  report::message::{
    highlight::{Highlight, HighlightType},
    line::HighlightedLine,
    Message, MessageMeta, MessageType,
  },
};
use ast::{FunctionNode, SemifiedProgram};
use decl::{ProgDeclMap, ScopedNameResolver};
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
  pub fn resolve(
    declmap: &ProgDeclMap,
    program: &HashMap<ModulePath, (Tree, UseMap)>,
  ) -> SemifiedProgram {
    let mut semified = SemifiedProgram::new();

    for (module, (tree, usemap)) in program {
      for node in &tree.nodes {
        match node {
          Node::VarDecl { typ, val, mutable } => {
            let constexpr = if let Some(constexpr) = val.as_const() {
              constexpr
            } else {
              let highlight = Highlight::new(
                typ.name.column,
                typ.name.len,
                HighlightType::Focus,
              );
              let line = HighlightedLine::from_src(&tree.source, typ.name.line)
                .with_highlight(highlight);
              Message::new(
                "Global variables must have constant initial values",
                MessageType::Error,
              )
              .with_meta(MessageMeta::FileLocation(
                tree.path.clone(),
                typ.name.line,
                typ.name.column,
              ))
              .with_line(line)
              .report_and_exit(1);
            };

            semified.decl_var(
              module,
              typ.name.clone(),
              typ.definite(),
              constexpr,
              *mutable,
            )
          }
          Node::FnDecl {
            attributes,
            name,
            params,
            ret_type,
            body,
          } => {
            let resolver = ScopedNameResolver::new(declmap, usemap, params);
          }
          Node::StructDecl { name, fields } => {
            // Need to verify that the types referenced in the fields point to somewhere and are not cyclycal
            // But anyways, we can't even parse dot syntax now, let alone semify structs
            Message::compiler_bug("struct is not yet implemented")
              .report_and_exit(1)
          }
          _ => {}
        }
      }
    }

    semified
  }

  fn resolve_body(
    body: &[Node],
    declmap: &ProgDeclMap,
    usemap: &UseMap,
    params: &[TypedNameLI<GlobalIdentifier>],
  ) -> Vec<FunctionNode> {
    let mut resolver = ScopedNameResolver::new(declmap, usemap, params);
    let mut resolved_body = vec![];
    for node in body {
      match node {
        Node::Expression(expr) => resolved_body.extend(
          expr.minify().into_iter().map(|expr_side_effect| {
            FunctionNode::SideEffect(expr_side_effect.resolve(&resolver))
          }),
        ),
        Node::VarDecl { typ, val, mutable } => {
          let resolved_val = val.resolve(&resolver);
          let 
          resolver.add_var(typ.clone().into());
          resolved_body.push(FunctionNode::Assignment { target: ScopedIdentifier::Local(()), val: () })
        },
        Node::Assignment { target, op, val } => {

        },
        Node::Return(_) => todo!(),
        _ => Message::compiler_bug("Unexpected node in function body")
          .report_and_exit(1),
      }
    }
    resolved_body
  }
}
