mod format;
mod grammar;
mod interface;
mod output;

use format::PackMeta;
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

  let pack = filesystem.root().borrow_mut().file("pack.mcmeta");
  let data = filesystem.root().borrow_mut().subdirectory("data");
  let namespace = data.borrow_mut().subdirectory(&config.datapack.name);
  let function = namespace.borrow_mut().subdirectory("function");
  let poc = function.borrow_mut().file("poc.mcfunction");

  let pack_content =
    serde_json::to_string_pretty(&PackMeta::new(&config.datapack.description))
      .expect("Could not serialize pack.mcmeta");

  pack.borrow_mut().write(pack_content.as_bytes());
  poc.borrow_mut().write("say Hello, World!".as_bytes());
}
