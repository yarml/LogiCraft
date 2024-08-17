use colored::{Color, ColoredString, Colorize};
use std::collections::{HashMap, HashSet};

use super::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightType {
  Suggestion,
  Helper,
  Focus,
}

#[derive(Debug, Clone)]
pub struct Highlight {
  start: usize,
  end: usize,
  typ: HighlightType,
  label: Option<String>,
}

struct HighlightSet {
  highlights: HashMap<usize, HighlightType>,
}

#[derive(Debug, Clone)]
struct DetailSet {
  details: HashMap<usize, DetailSpot>,
}

struct FinalizeCache {
  displayed_pipes: HashMap<usize, HashSet<HighlightType>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailSpot {
  Free,
  Padding,
  Character(char, HighlightType),
  Head(char, HighlightType),
}

pub(super) struct HighlightContext {
  markers: Vec<HighlightSet>,
  details: Vec<DetailSet>,
}

impl Highlight {
  pub fn new(start: usize, len: usize, typ: HighlightType) -> Self {
    Self {
      start,
      end: start + len,
      typ,
      label: None,
    }
  }

  pub fn with_label(mut self, label: &str) -> Self {
    self.add_label(label);
    self
  }

  pub fn add_label(&mut self, label: &str) {
    self.label = Some(String::from(label));
  }
}

impl HighlightType {
  pub fn marker(&self) -> char {
    match self {
      HighlightType::Suggestion => '~',
      HighlightType::Helper => '-',
      HighlightType::Focus => '^',
    }
  }
  pub fn color(&self) -> Color {
    match self {
      HighlightType::Suggestion => Color::Green,
      HighlightType::Helper => Color::Yellow,
      HighlightType::Focus => Color::Red,
    }
  }
}

impl HighlightContext {
  pub fn new() -> Self {
    Self {
      markers: Vec::new(),
      details: Vec::new(),
    }
  }
  pub fn add_highlight(&mut self, highlight: Highlight) {
    let Highlight {
      start,
      end,
      typ,
      label,
    } = highlight;
    let free_marker = self
      .markers
      .iter_mut()
      .find(|hs| hs.is_free(start, end, typ));

    if let Some(marker) = free_marker {
      marker.set(start, end, typ);
    } else {
      let mut new_marker = HighlightSet::new();
      new_marker.set(start, end, typ);
      self.markers.push(new_marker);
    }

    if let Some(label) = label {
      let free_detail = self
        .details
        .iter_mut()
        .find(|ds| ds.is_free(start, label.len()));

      if let Some(detail) = free_detail {
        detail.put_string(start, &label, typ);
      } else {
        let mut new_detail = DetailSet::new();
        new_detail.put_string(start, &label, typ);
        self.details.push(new_detail);
      }
    }
  }

  pub fn finalize(&self, padding: usize, margin: ColoredString) -> String {
    assert!(margin.len() == 1);
    let mut cache = FinalizeCache::new();
    let mut result = String::new();
    let header = format!("{:padding$} {margin} ", "");

    for marker in self.markers.iter() {
      let max = marker.max();
      if max == 0 {
        continue;
      }
      let mut line_result = header.clone();
      line_result.reserve(max);
      for pos in 0..=max {
        let typ = marker.get(pos);
        let marker = typ.map_or(' ', |t| t.marker());
        let color = typ.map_or(Color::White, |t| t.color());
        line_result.push_str(&format!("{}", marker.to_string().color(color)));
      }
      result.push_str(&format!("{}\n", line_result));
    }

    for (line, detail) in self.details.iter().enumerate() {
      let max = detail.max();
      if max == 0 {
        continue;
      }
      let mut line_result = header.clone();
      line_result.reserve(max);
      for pos in 0..=max {
        let spot = detail.get(pos);
        if spot.looks_empty() {
          let remaining_heads = self.remaining_heads(pos, line);
          if !remaining_heads.is_empty() {
            let typ = cache.next_pipe(pos, &remaining_heads);
            line_result.push_str(&format!("{margin}").color(typ.color()))
          }
        } else {
          let (c, color) = match spot {
            DetailSpot::Free => (' ', Color::White),
            DetailSpot::Padding => (' ', Color::White),
            DetailSpot::Character(c, t) => (c, t.color()),
            DetailSpot::Head(c, t) => (c, t.color()),
          };
          line_result.push_str(&format!("{}", c.to_string().color(color)));
        }
      }
      result.push_str(&format!("{}\n", line_result));
    }

    result
  }

  fn remaining_heads(
    &self,
    column: usize,
    line: usize,
  ) -> HashSet<HighlightType> {
    let mut result = HashSet::new();
    for detail in self.details.iter().skip(line) {
      if let DetailSpot::Head(_, typ) = detail.get(column) {
        result.insert(typ);
      }
    }
    result
  }
}

impl HighlightSet {
  pub fn new() -> Self {
    HighlightSet {
      highlights: HashMap::new(),
    }
  }

  fn set(&mut self, start: usize, end: usize, typ: HighlightType) {
    for i in start..end {
      self.highlights.insert(i, typ);
    }
  }

  fn get(&self, pos: usize) -> Option<HighlightType> {
    self.highlights.get(&pos).copied()
  }

  fn max(&self) -> usize {
    *self.highlights.keys().max().unwrap_or(&0)
  }

  fn is_free(&self, start: usize, end: usize, typ: HighlightType) -> bool {
    for i in start..end {
      if self.highlights.get(&i).map_or(false, |t| *t != typ) {
        return false;
      }
    }
    true
  }
}

impl DetailSet {
  pub fn new() -> Self {
    DetailSet {
      details: HashMap::new(),
    }
  }

  fn set(&mut self, pos: usize, detail: DetailSpot) {
    self.details.insert(pos, detail);
  }

  fn put_string(&mut self, pos: usize, string: &str, typ: HighlightType) {
    if string.is_empty() {
      return;
    }
    if pos >= 1 {
      self.set(pos - 1, DetailSpot::Padding);
    }
    self.set(pos + string.len(), DetailSpot::Padding);
    let mut chars = string.chars();
    let head = chars.next().unwrap();
    self.set(pos, DetailSpot::Head(head, typ));
    for (i, c) in chars.enumerate() {
      self.set(pos + i + 1, DetailSpot::Character(c, typ));
    }
  }

  fn get(&self, pos: usize) -> DetailSpot {
    self
      .details
      .get(&pos)
      .copied()
      .map_or(DetailSpot::Free, |d| d)
  }

  fn max(&self) -> usize {
    *self.details.keys().max().unwrap_or(&0)
  }

  fn is_free(&self, start: usize, len: usize) -> bool {
    let start = if start < 1 { 0 } else { start - 1 };
    let end = start + len + 1;
    for i in start..end {
      if self.get(i) != DetailSpot::Free {
        return false;
      }
    }
    true
  }
}

impl DetailSpot {
  fn looks_empty(&self) -> bool {
    match self {
      DetailSpot::Free => true,
      DetailSpot::Padding => true,
      _ => false,
    }
  }
}

impl FinalizeCache {
  fn new() -> Self {
    Self {
      displayed_pipes: HashMap::new(),
    }
  }
  fn next_pipe(
    &mut self,
    column: usize,
    possible_values: &HashSet<HighlightType>,
  ) -> HighlightType {
    let displayed_values =
      self.displayed_pipes.entry(column).or_insert(HashSet::new());
    let mut possibles_nexts = possible_values
      .iter()
      .filter(|v| !displayed_values.contains(v));

    if let Some(next) = possibles_nexts.next() {
      displayed_values.insert(*next);
      *next
    } else {
      displayed_values.clear();
      *possible_values.iter().next().unwrap_or_else(|| {
        Message::compiler_bug(
          "next_pipe cannot operate on empty set of possibilities",
        )
        .report_and_exit(1)
      })
    }
  }
}
