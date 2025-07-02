use std::ops::{Deref, DerefMut};

use chrono::{DateTime, Local};
use ratatui::prelude::{Color, Stylize};
use ratatui::text::{Line, Span};

impl Default for Timestamp {
  fn default() -> Self {
    Self(Local::now())
  }
}
impl Deref for Timestamp {
  type Target = DateTime<Local>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl DerefMut for Timestamp {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl AsRef<Timestamp> for Timestamp {
  fn as_ref(&self) -> &Timestamp {
    self
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Timestamp(DateTime<Local>);

impl Timestamp {
  pub fn as_spans(&self) -> [Span<'static>; 3] {
    [
      Span::raw(ratatui::symbols::line::DOUBLE_VERTICAL_LEFT).fg(Color::DarkGray),
      Span::raw(self.0.format("%H:%M:%S").to_string()).fg(Color::White),
      Span::raw(ratatui::symbols::line::DOUBLE_VERTICAL_RIGHT).fg(Color::DarkGray),
    ]
  }
  pub fn now() -> Self {
    Self::default()
  }
}

impl From<Timestamp> for Line<'static> {
  fn from(value: Timestamp) -> Self {
    Line::from_iter(value.as_spans())
  }
}

impl<'s> From<&'s Timestamp> for Line<'s> {
  fn from(value: &'s Timestamp) -> Self {
    Line::from_iter(value.as_spans())
  }
}
