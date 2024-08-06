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
struct Args {
  /// Path to root directory of project
  #[arg(short = 's', default_value = "./")]
  source: PathBuf,
  /// Path to output directory/zip
  #[arg(short = 'o', default_value = "./out.zip")]
  output: PathBuf,
  #[arg(short = 't', default_value_t = OutputType::Directory)]
  output_type: OutputType,
}

#[derive(Debug)]
pub struct ArgsConfig {
  pub source: PathBuf,
  pub output: PathBuf,
  pub output_type: OutputType,
}

pub fn getargs() -> ArgsConfig {
  let args = Args::parse();
  ArgsConfig {
    source: args.source,
    output: args.output,
    output_type: args.output_type,
  }
}
