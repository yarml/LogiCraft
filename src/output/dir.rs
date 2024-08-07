use super::{OutputDirectory, OutputFile, OutputFilesystem};
use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};

pub struct DirOutputFilesystem {
  root: Rc<RefCell<DirOutputDirectory>>,
}

pub struct DirOutputDirectory {
  path: PathBuf,
  entries: HashMap<String, DirOutputEntry>,
}
pub struct DirOutputFile {
  path: PathBuf,
}

enum DirOutputEntry {
  File(Rc<RefCell<DirOutputFile>>),
  Directory(Rc<RefCell<DirOutputDirectory>>),
}

impl DirOutputFilesystem {
  pub fn new(destination: &PathBuf, force: bool) -> Option<Self> {
    if destination.exists() && !force {
      return None;
    }

    Some(Self {
      root: Rc::new(RefCell::new(DirOutputDirectory::new(&destination))),
    })
  }
}
impl DirOutputDirectory {
  pub fn new(path: &PathBuf) -> Self {
    if path.exists() {
      let md = fs::metadata(&path).expect(&format!(
        "Could not read existing directory's metadata: {path:?}"
      ));
      if md.is_dir() {
        fs::remove_dir_all(&path)
          .expect(&format!("Could not remove existing directory: {path:?}"));
      } else if md.is_file() || md.is_symlink() {
        fs::remove_file(&path)
          .expect(&format!("Could not remove existing file: {path:?}"));
      }
    }
    fs::create_dir(&path)
      .expect(&format!("Could not create directory: {path:?}"));
    Self {
      path: path.clone(),
      entries: HashMap::new(),
    }
  }
}
impl DirOutputFile {
  pub fn new(path: &PathBuf) -> Self {
    Self { path: path.clone() }
  }
}

impl OutputFilesystem for DirOutputFilesystem {
  fn root(&self) -> Rc<RefCell<dyn OutputDirectory>> {
    self.root.clone()
  }
}

impl OutputDirectory for DirOutputDirectory {
  fn subdirectory(&mut self, name: &str) -> Rc<RefCell<dyn OutputDirectory>> {
    let path = self.path.join(name);
    match self.entries.entry(String::from(name)).or_insert_with(|| {
      DirOutputEntry::Directory(Rc::new(RefCell::new(DirOutputDirectory::new(
        &path,
      ))))
    }) {
      DirOutputEntry::File(_) => panic!("Tried creating a directory when previously a file was created at the same path: {path:?}"),
      DirOutputEntry::Directory(dir) => dir.clone(),
    }
  }

  fn file(&mut self, name: &str) -> Rc<RefCell<dyn OutputFile>> {
    let path = self.path.join(name);
    match self.entries.entry(String::from(name)).or_insert_with(|| {
      DirOutputEntry::File(Rc::new(RefCell::new(DirOutputFile::new(&path))))
    }) {
      DirOutputEntry::File(file) => file.clone(),
      DirOutputEntry::Directory(_) => panic!("Tried creating a file when previously a directory was created at the same path: {path:?}"),
    }
  }
}

impl OutputFile for DirOutputFile {
  fn write(&mut self, data: &[u8]) {
    fs::write(self.path.clone(), data)
      .expect(&format!("Could not write to file: {:?}", self.path));
  }
}
