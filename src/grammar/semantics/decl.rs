use super::{module::ModulePath, usemap::UseMap};
use crate::{
  grammar::{
    builtins::BuiltinType,
    identifier::{
      GlobalIdentifier, Name, ScopedIdentifier, Type, UnscopedIdentifier,
    },
    parser::ast::{Node, TypedNameLI},
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

pub struct LocalDeclMap {
  decls: HashMap<Name, TypedNameLI<GlobalIdentifier>>,
}

pub struct ScopedNameResolver<'a, 'b> {
  glob_decls: &'a ProgDeclMap,
  usemap: &'b UseMap,
  scope_decls: LocalDeclMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalDecl {
  pub id: GlobalIdentifier,
  pub typ: GlobalDeclType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalDeclType {
  Variable {
    typ: WithLineInfo<Type<GlobalIdentifier>>,
    mutable: bool,
  },
  Function {
    args: Vec<WithLineInfo<Type<GlobalIdentifier>>>,
    ret: WithLineInfo<Type<GlobalIdentifier>>,
  },
  Struct(Vec<TypedNameLI<GlobalIdentifier>>),
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
    args: Vec<WithLineInfo<Type<GlobalIdentifier>>>,
    ret: WithLineInfo<Type<GlobalIdentifier>>,
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
    typ: WithLineInfo<Type<GlobalIdentifier>>,
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
    fields: Vec<TypedNameLI<GlobalIdentifier>>,
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
impl LocalDeclMap {
  pub fn from_params(params: &[TypedNameLI<GlobalIdentifier>]) -> Self {
    let mut instance = Self {
      decls: HashMap::new(),
    };
    for param in params {
      instance.add_var(param.clone());
    }
    instance
  }
  pub fn add_var(&mut self, vardecl: TypedNameLI<GlobalIdentifier>) {
    self.decls.insert(vardecl.name.value.clone(), vardecl);
  }

  pub fn lookup(&self, name: &str) -> Option<&TypedNameLI<GlobalIdentifier>> {
    self.decls.get(name)
  }
}
impl<'a, 'b> ScopedNameResolver<'a, 'b> {
  pub fn new(
    declmap: &'a ProgDeclMap,
    usemap: &'b UseMap,
    params: &[TypedNameLI<GlobalIdentifier>],
  ) -> Self {
    Self {
      glob_decls: declmap,
      usemap,
      scope_decls: LocalDeclMap::from_params(params),
    }
  }

  pub fn add_var(&mut self, vardecl: TypedNameLI<GlobalIdentifier>) {
    self.scope_decls.add_var(vardecl)
  }

  pub fn resolve(&self, id: &UnscopedIdentifier) -> ScopedIdentifier {
    if id.parts.len() == 0 {
      Message::tmp("Brother, I really don't wanna deal with this now")
        .report_and_exit(1);
    }
    // If singular, this can then be local, otherwise it has to be global
    if id.is_singular() {
      let name = &id.parts[0].value;
      if let Some(resolved) = self.scope_decls.lookup(name) {
        return ScopedIdentifier::Local(resolved.clone());
      }
    }
    if id.root {
      let glob_id = id.as_root();
      if let Some(glob_decl) = self.glob_decls.lookup(&glob_id) {
        return ScopedIdentifier::Global(glob_decl.clone());
      } else {
        Message::tmp(&format!(
          "Full identifier referencing an unknown target: {glob_id}",
        ))
        .report_and_exit(1)
      }
    }
    let first_name = &id.parts[0].value;
    if let Some(glob_id) = self.usemap.lookup(&first_name) {
      let mut glob_id = glob_id.clone();
      glob_id.push_names(
        &id
          .parts
          .iter()
          .map(|winfo| winfo.value.clone())
          .collect::<Vec<_>>()[1..],
      );
      if let Some(glob_decl) = self.glob_decls.lookup(&glob_id) {
        return ScopedIdentifier::Global(glob_decl.clone());
      } else {
        Message::tmp(&format!(
          "Identifier referencing an unknown target: {glob_id}",
        ))
        .report_and_exit(1)
      }
    } else {
      Message::tmp(&format!("Identifier referencing an unused: {first_name}",))
        .report_and_exit(1)
    }
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
