use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub datapack: DatapackConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatapackConfig {
  /// Datapack name
  pub name: String,
  /// Datapack version
  pub version: String,
  /// LogiCraft Compiler version
  pub format: usize,
}

pub fn getconfig(path: PathBuf) -> Config {
  let config_raw = fs::read_to_string(path.clone())
    .expect(&format!("Could not read configuration file: {path:?}"));
  toml::from_str(&config_raw).expect("Could not parse configuration file")
}
