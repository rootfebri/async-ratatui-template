use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Stylize, Widget};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph};

use crate::ui::blk;

#[derive(Debug, Clone)]
pub struct Statistic {
  cur: Arc<AtomicUsize>,
  max: Arc<AtomicUsize>,
  label: String,
}

impl Default for Statistic {
  fn default() -> Self {
    Self::new("Progress")
  }
}

impl Statistic {
  pub fn new(label: impl Into<String>) -> Self {
    Self {
      cur: Arc::new(AtomicUsize::new(0)),
      max: Arc::new(AtomicUsize::new(100)),
      label: label.into(),
    }
  }

  pub fn set_current(&self, value: usize) {
    self.cur.store(value, Ordering::Relaxed);
  }

  pub fn set_max(&self, value: usize) {
    self.max.store(value, Ordering::Relaxed);
  }

  pub fn increment(&self) {
    self.cur.fetch_add(1, Ordering::Relaxed);
  }

  pub fn get_current(&self) -> usize {
    self.cur.load(Ordering::Relaxed)
  }

  pub fn get_max(&self) -> usize {
    self.max.load(Ordering::Relaxed)
  }

  pub fn get_percentage(&self) -> f64 {
    let current = self.get_current() as f64;
    let max = self.get_max() as f64;
    if max == 0.0 { 0.0 } else { (current / max * 100.0).min(100.0) }
  }

  pub fn is_complete(&self) -> bool {
    self.get_current() >= self.get_max()
  }

  fn draw_title(&self) -> Line {
    let current = self.get_current();
    let max = self.get_max();
    let percentage = self.get_percentage();

    let spans = vec![
      Span::raw(" "),
      Span::raw(&self.label).fg(Color::Cyan),
      Span::raw(": "),
      Span::raw(format!("{current}")).fg(Color::Yellow),
      Span::raw("/"),
      Span::raw(format!("{max}")).fg(Color::White),
      Span::raw(" ("),
      Span::raw(format!("{percentage:.1}%")).fg(if self.is_complete() { Color::Green } else { Color::Blue }),
      Span::raw(") "),
    ];

    Line::from(spans)
  }
}

impl Widget for &Statistic {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let [title_area, gauge_area] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

    // Render title
    Paragraph::new(self.draw_title()).render(title_area, buf);

    // Render gauge
    let ratio = self.get_percentage() / 100.0;
    let gauge_color = if self.is_complete() {
      Color::Green
    } else if ratio > 0.7 {
      Color::Yellow
    } else {
      Color::Blue
    };

    Gauge::default()
      .block(blk())
      .gauge_style(gauge_color)
      .ratio(ratio)
      .render(gauge_area, buf);
  }
}
