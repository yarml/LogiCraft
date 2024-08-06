mod args;
mod config;

use args::getargs;
use config::getconfig;

fn main() {
  let args = getargs();
  let mut config_path = args.source.clone();
  config_path.push("lc.toml");
  let config = getconfig(config_path);

  println!("Args: {args:?}");
  println!("Config: {config:?}")
}
