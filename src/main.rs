mod control;
mod format;
mod grammar;
mod output;
mod pipeline;
mod report;

use control::cli::{getargs, OutputType};
use control::config::getconfig;
use format::pack::PackMeta;
use output::dir::DirOutputFilesystem;
use output::OutputFilesystem;
use pipeline::Pipeline;
use report::message::{Message, MessageType};

fn main() {
  let args = getargs();
  let config = getconfig(args.source.join("lc.toml"));
  let output_extension = match args.output_type {
    OutputType::Directory => "",
    OutputType::Zip => ".zip",
  };

  // Lex main.lc
  let src_path = args.source.join("src");
  let pipeline = Pipeline::new(&src_path);
  pipeline.run();

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
    None => Message::new(
      "Destination already exists. Use -f to overwrite.",
      MessageType::Error,
    )
    .report_and_exit(1),
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
