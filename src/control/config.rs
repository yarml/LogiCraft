use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

use crate::report::message::{Message, MessageType};

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
  /// Datapack description
  pub description: String,
}

pub fn getconfig(path: PathBuf) -> Config {
  let config_raw = fs::read_to_string(path.clone())
    .unwrap_or_else(|err| Message::input_error(err, &path).report_and_exit(1));
  toml::from_str(&config_raw).unwrap_or_else(|err| {
    Message::new(
      &format!("Could not parse `{path:?}`: {}", err.to_string()),
      MessageType::Error,
    )
    .report_and_exit(1)
  })
}
