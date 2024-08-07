use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputType {
  Directory,
  Zip,
}

impl ToString for OutputType {
  fn to_string(&self) -> String {
    match self {
      OutputType::Directory => String::from("directory"),
      OutputType::Zip => String::from("zip"),
    }
  }
}

#[derive(Debug, Parser)]
#[command(version = "1.0", about = "LogiCraft Compiler", long_about= None)]
pub struct Args {
  /// Path to root directory of project
  #[arg(short = 's', default_value = "./")]
  pub source: PathBuf,
  /// Path to output directory/zip
  #[arg(short = 'o', default_value = "./out/")]
  pub output: PathBuf,
  #[arg(short = 't', default_value_t = OutputType::Directory)]
  pub output_type: OutputType,
  #[arg(short = 'f', default_value_t = false)]
  pub force: bool,
}

pub fn getargs() -> Args {
  Args::parse()
}
