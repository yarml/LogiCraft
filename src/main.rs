mod grammar;
mod interface;
mod output;

use interface::cli::{getargs, OutputType};
use interface::config::getconfig;
use output::dir::DirOutputFilesystem;
use output::OutputFilesystem;

fn main() {
  let args = getargs();
  let config = getconfig(args.source.join("lc.toml"));
  let output_extension = match args.output_type {
    OutputType::Directory => "",
    OutputType::Zip => ".zip",
  };

  let output_name = format!(
    "{name}-{version}-{format}{ext}",
    name = config.datapack.name,
    version = config.datapack.version,
    format = config.datapack.format,
    ext = output_extension
  );
  let destination = args.output.join(&output_name);

  let filesystem = match DirOutputFilesystem::new(&destination, args.force) {
    Some(fs) => fs,
    None => {
      panic!("Destination already exists. Use -f to overwrite.");
    }
  };

  filesystem
    .root()
    .borrow_mut()
    .file("pack.mcmeta")
    .borrow_mut()
    .write("Hello, World!".as_bytes());
}
