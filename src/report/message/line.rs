use super::highlight::{Highlight, HighlightContext};
use colored::{Color, Colorize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
  Source,
  Suggestion,
}

#[derive(Debug, Clone)]
pub struct HighlightedLine {
  num: usize,
  line: String,
  typ: LineType,
  highlights: Vec<Highlight>,
}

impl LineType {
  pub fn margin(&self) -> char {
    match self {
      LineType::Source => '|',
      LineType::Suggestion => '#',
    }
  }
  pub fn color(&self) -> Color {
    match self {
      LineType::Source => Color::Blue,
      LineType::Suggestion => Color::Green,
    }
  }
}

impl HighlightedLine {
  pub fn new(num: usize, line: &str, typ: LineType) -> Self {
    Self {
      num,
      line: String::from(line),
      typ,
      highlights: Vec::new(),
    }
  }
  pub fn from_src(source: &str, num: usize) -> Self {
    Self::new(num, source.lines().nth(num - 1).unwrap(), LineType::Source)
  }

  pub fn with_highlight(mut self, highlight: Highlight) -> Self {
    self.highlights.push(highlight);
    self
  }

  pub(super) fn prepare(&self) -> String {
    let margin = self.typ.margin();
    let margin_color = self.typ.color();
    let margin_colored = format!("{margin}").color(margin_color);

    let num_len = (self.num as f64).log10().ceil() as usize;
    let num_padding = if num_len > 3 { num_len } else { 3 };

    let source_line = format!(
      "{num:>num_padding$} {margin_colored} {line}\n",
      num = self.num,
      line = self.line
    );

    let mut hcontext = HighlightContext::new();
    for h in &self.highlights {
      hcontext.add_highlight(h.clone());
    }
    let highlight_lines = hcontext.finalize(num_padding, margin_colored);

    format!("{source_line}{highlight_lines}")
  }
}
