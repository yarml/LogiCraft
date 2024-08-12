use colored::Colorize;
use std::{path::PathBuf, process};

pub fn report_and_exit(
  line: &str,
  file: &PathBuf,
  line_num: usize,
  column: usize,
  len: usize,
  message: &str,
  details: Option<&str>,
  ec: i32,
) -> ! {
  let line_num_str = format!("{}", line_num).cyan().bold();
  let bar = "|".yellow();
  let error_source =
    format!("{file}:{line_num}:{column}:", file = file.to_string_lossy())
      .yellow()
      .bold();

  let column = column - 1;

  let line_pre_offending = &line[..column];
  let line_offending = &line[column..column + len].red().bold();
  let line_post_offending = &line[column + len..];

  eprintln!("{error_source}");
  eprintln!("{line_num_str:>4} {bar} {line_pre_offending}{line_offending}{line_post_offending}",);
  eprintln!(
    "{filler:>4} {bar} {filler:<column$}{arrow:~<len$} {message}",
    filler = "",
    arrow = "^".red().bold(),
    message = message.red(),
  );

  if let Some(details) = details {
    eprintln!("{}", details.bold());
  }

  process::exit(ec);
}
