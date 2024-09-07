use super::module::ModulePath;
use crate::{
  grammar::{
    builtins::BuiltinType,
    identifier::{GlobalIdentifier, Name, Type},
    parser::ast::{Node, TypedName},
  },
  pipeline::Tree,
  report::{location::WithLineInfo, message::Message},
};
use std::{collections::HashMap, fmt::Display};

// ProgDeclMap: Keeps track of all global declarations within all modules.
// This includes the module, name, and type.
#[derive(Debug, Clone)]
pub struct ProgDeclMap {
  decls: HashMap<GlobalIdentifier, GlobalDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDecl {
  pub id: GlobalIdentifier,
  pub typ: GlobalDeclType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalDeclType {
  Variable {
    typ: WithLineInfo<Type>,
    mutable: bool,
  },
  Function {
    args: Vec<WithLineInfo<Type>>,
    ret: WithLineInfo<Type>,
  },
  Struct(Vec<TypedName>),
}

impl ProgDeclMap {
  pub fn new() -> Self {
    ProgDeclMap {
      decls: HashMap::new(),
    }
  }

  pub fn add_decl(&mut self, id: GlobalIdentifier, typ: GlobalDeclType) {
    self.decls.insert(id.clone(), GlobalDecl { id, typ });
  }

  pub fn add_fn(
    &mut self,
    module: &ModulePath,
    name: WithLineInfo<Name>,
    args: Vec<WithLineInfo<Type>>,
    ret: WithLineInfo<Type>,
  ) {
    let id = GlobalIdentifier {
      module: module.clone(),
      name,
    };
    self.add_decl(id, GlobalDeclType::Function { args, ret });
  }

  pub fn add_var(
    &mut self,
    module: &ModulePath,
    name: WithLineInfo<Name>,
    typ: WithLineInfo<Type>,
    mutable: bool,
  ) {
    let id = GlobalIdentifier {
      module: module.clone(),
      name,
    };
    self.add_decl(id, GlobalDeclType::Variable { typ, mutable });
  }

  pub fn add_struct(
    &mut self,
    module: &ModulePath,
    name: WithLineInfo<Name>,
    fields: Vec<TypedName>,
  ) {
    let id = GlobalIdentifier {
      module: module.clone(),
      name,
    };
    self.add_decl(id, GlobalDeclType::Struct(fields));
  }

  pub fn add_module(&mut self, module: &ModulePath, tree: &Tree) {
    for node in &tree.nodes {
      match node {
        Node::VarDecl { typ, mutable, .. } => {
          if typ.typ.is_none() {
            Message::compiler_bug(
              "Top level variable has no explicit type attached",
            )
            .report_and_exit(1);
          }
          self.add_var(
            module,
            typ.name.clone(),
            typ.typ.clone().unwrap(),
            *mutable,
          );
        }
        Node::FnDecl {
          name,
          params,
          ret_type,
          ..
        } => {
          let void_type =
            name.clone().map(|_| Type::Builtin(BuiltinType::Void));
          self.add_fn(
            module,
            name.clone(),
            params
              .iter()
              .map(|typed_name| typed_name.typ.clone())
              .collect(),
            ret_type.clone().map_or(void_type, |typ| typ),
          )
        }
        Node::StructDecl { name, fields } => {
          self.add_struct(module, name.clone(), fields.clone())
        }
        Node::ModDecl(_) | Node::UseDecl(_) => {}
        _ => Message::compiler_bug("Unexpected node at top tree level")
          .report_and_exit(1),
      }
    }
  }

  pub fn lookup(&self, id: &GlobalIdentifier) -> Option<&GlobalDecl> {
    self.decls.get(id)
  }
}

impl Display for ProgDeclMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for decl in self.decls.values() {
      write!(f, "{decl}\n")?;
    }
    Ok(())
  }
}

impl Display for GlobalDecl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} -> {}", self.id, self.typ)
  }
}

impl Display for GlobalDeclType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GlobalDeclType::Variable { typ, mutable } => {
        let prefix = if *mutable { "var" } else { "const" };
        write!(f, "{prefix} {typ}")
      }
      GlobalDeclType::Function { args, ret } => {
        let args_types = args
          .iter()
          .map(|typ| typ.to_string())
          .collect::<Vec<_>>()
          .join(", ");
        write!(f, "fn({args_types}) -> {ret}")
      }
      GlobalDeclType::Struct(fields) => {
        let struct_types = fields
          .iter()
          .map(|field| field.typ.to_string())
          .collect::<Vec<_>>()
          .join(", ");
        write!(f, "struct {{{struct_types}}}")
      }
    }
  }
}
