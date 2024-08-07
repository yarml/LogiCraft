pub mod dir;
pub mod zip;

use std::{cell::RefCell, rc::Rc};

pub trait OutputFilesystem {
  fn root(&self) -> Rc<RefCell<dyn OutputDirectory>>;
}

pub trait OutputDirectory {
  fn subdirectory(&mut self, name: &str) -> Rc<RefCell<dyn OutputDirectory>>;
  fn file(&mut self, name: &str) -> Rc<RefCell<dyn OutputFile>>;
}

pub trait OutputFile {
  fn write(&mut self, data: &[u8]);
}
