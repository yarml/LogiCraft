pub mod highlight;
pub mod line;

use colored::{Color, Colorize};
use line::HighlightedLine;
use std::{path::PathBuf, process};

#[derive(Debug, Clone)]
pub struct Message {
  typ: MessageType,
  message: String,
  meta: Vec<MessageMeta>,
  lines: Vec<HighlightedLine>,
  notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
  Help,
  Warning,
  Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitBehavior {
  AlwaysExit(i32),
  ExitIfEntailed(i32),
  OnlyReport,
}

#[derive(Debug, Clone)]
pub enum MessageMeta {
  FileLocation(PathBuf, usize, usize),
}

impl Message {
  pub fn new(message: &str, typ: MessageType) -> Self {
    Self {
      message: String::from(message),
      typ,
      lines: Vec::new(),
      meta: Vec::new(),
      notes: Vec::new(),
    }
  }

  pub fn with_note(mut self, note: &str) -> Self {
    self.notes.push(String::from(note));
    self
  }

  pub fn with_meta(mut self, meta: MessageMeta) -> Self {
    self.meta.push(meta);
    self
  }

  pub fn with_line(mut self, line: HighlightedLine) -> Self {
    self.lines.push(line);
    self
  }

  pub fn report(&self, exit_behavior: ExitBehavior) -> bool {
    println!(
      "{}: {}",
      self.typ.header().color(self.typ.color()).bold(),
      self.message
    );

    for meta in &self.meta {
      println!("{}", meta.prepare());
    }

    for line in &self.lines {
      println!("{}", line.prepare());
    }

    let note_label = "note".cyan().bold();
    for note in &self.notes {
      println!("   {note_label}: {note}\n");
    }

    match exit_behavior {
      ExitBehavior::AlwaysExit(code) | ExitBehavior::ExitIfEntailed(code)
        if self.typ.entails_exit() =>
      {
        process::exit(code)
      }
      _ => self.typ.entails_exit(),
    }
  }

  pub fn report_and_exit(&self, code: i32) -> ! {
    self.report(ExitBehavior::AlwaysExit(code));
    unreachable!()
  }
}

impl MessageType {
  pub fn header(&self) -> &'static str {
    match self {
      MessageType::Help => "help",
      MessageType::Warning => "warning",
      MessageType::Error => "error",
    }
  }

  pub fn color(&self) -> Color {
    match self {
      MessageType::Help => Color::Green,
      MessageType::Warning => Color::Yellow,
      MessageType::Error => Color::Red,
    }
  }
  pub fn entails_exit(&self) -> bool {
    match self {
      MessageType::Error => true,
      _ => false,
    }
  }
}

impl MessageMeta {
  fn prepare(&self) -> String {
    match self {
      MessageMeta::FileLocation(path, line, col) => {
        let path = path.to_string_lossy().to_string();
        format!(
          "   {arrow} {path}:{line}:{col}",
          arrow = "-->".blue().bold()
        )
      }
    }
  }
}
