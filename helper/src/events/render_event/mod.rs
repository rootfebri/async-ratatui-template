use std::time::{Duration, Instant};

use ratatui::text::{Line, Span};

macro_rules! impl_variants {
    ($vi:vis, $variant:ident, $event:ident$(($($parname:ident:$partype:ty),*))? $(,)?) => {
      impl $crate::RenderEvent {
        $vi fn $variant($($($parname: $partype),+)?) -> Self {
          Self {kind: $crate::EventKind::$event$(($($parname),+))?, event_time: Instant::now()}
        }
      }
    };
}

impl_variants!(pub, canceled, Canceled);
impl_variants!(pub, render, Render);
impl_variants!(pub, no_ops, NoOps);
impl_variants!(pub, handled, Handled);
impl_variants!(pub, error, Error(v: Span<'static>));
impl_variants!(pub, warn, Warn(v: Span<'static>));

mod event;
pub use event::EventKind;

impl Default for RenderEvent {
  fn default() -> Self {
    Self {
      kind: EventKind::NoOps,
      event_time: Instant::now(),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderEvent {
  pub kind: EventKind,
  event_time: Instant,
}

impl RenderEvent {
  pub fn as_handled(&mut self) {
    self.kind = EventKind::Handled;
    self.event_time = Instant::now();
  }
  pub fn modify_60fps(&mut self, new: Self) -> bool {
    if self.fps_60() {
      *self = new;
      true
    } else {
      false
    }
  }

  pub fn new(event: EventKind) -> Self {
    Self {
      kind: event,
      event_time: Instant::now(),
    }
  }

  pub fn read(self) -> EventKind {
    self.kind
  }

  pub fn event_time(&self) -> Instant {
    self.event_time
  }
  pub fn elapsed_since(&self) -> Duration {
    self.event_time.elapsed()
  }
  pub fn is_ms_ago(&self, rhs: u128) -> bool {
    self.event_time.elapsed().as_millis() > rhs
  }
  pub fn is_mc_ago(&self, rhs: u128) -> bool {
    self.event_time.elapsed().as_micros() > rhs
  }
  pub fn is_ns_ago(&self, rhs: u128) -> bool {
    self.event_time.elapsed().as_nanos() > rhs
  }

  pub fn fps_120(&self) -> bool {
    self.is_ms_ago(8)
  }
  pub fn fps_60(&self) -> bool {
    self.is_ms_ago(16)
  }
  pub fn fps_30(&self) -> bool {
    self.is_ms_ago(33)
  }
  pub fn fps_15(&self) -> bool {
    self.is_ms_ago(66)
  }
  pub fn is_already(&self, fps: u128) -> bool {
    self.is_ms_ago(1000 / fps)
  }

  pub fn event_as_line<I>(&self, spans: impl Into<Option<I>>) -> Line
  where
    I: IntoIterator<Item = Span<'static>>,
  {
    let span = match self.kind {
      EventKind::Error(ref span) | EventKind::Warn(ref span) => span.clone(),
      ref event => Span::from(event.to_string()),
    };

    if let Some(spans) = spans.into() {
      Line::from_iter(spans.into_iter().chain([span]))
    } else {
      Line::from(span)
    }
  }
}

impl From<std::io::Error> for RenderEvent {
  fn from(error: std::io::Error) -> Self {
    Self::new(EventKind::Error(Span::from(error.kind().to_string())))
  }
}
