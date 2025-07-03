use chrono::{DateTime, Local};
use ratatui::text::{Line, Span};

macro_rules! impl_variants {
    ($vi:vis, $variant:ident, $event:ident$(($($parname:ident:$partype:ty),*))? $(,)?) => {
      impl $crate::RenderEvent {
        $vi fn $variant($($($parname: $partype),+)?) -> Self {
          Self {kind: $crate::RenderKind::$event$(($($parname),+))?, event_time: Local::now()}
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
pub use event::RenderKind;

impl Default for RenderEvent {
  fn default() -> Self {
    Self {
      kind: RenderKind::NoOps,
      event_time: Local::now(),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderEvent {
  pub kind: RenderKind,
  pub event_time: DateTime<Local>,
}

impl RenderEvent {
  pub fn modif_render(&mut self) {
    *self = Self::render();
  }

  pub fn new(event: RenderKind) -> Self {
    Self {
      kind: event,
      event_time: Local::now(),
    }
  }

  pub fn read(self) -> RenderKind {
    self.kind
  }

  #[inline]
  pub fn is_frame(&self, fps: u16) -> bool {
    Local::now().timestamp_millis() - self.event_time.timestamp_millis() > 1000 / fps as i64
  }

  pub fn event_as_line<I>(&self, spans: impl Into<Option<I>>) -> Line
  where
    I: IntoIterator<Item = Span<'static>>,
  {
    let span = match self.kind {
      RenderKind::Error(ref span) | RenderKind::Warn(ref span) => span.clone(),
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
    Self::new(RenderKind::Error(Span::from(error.kind().to_string())))
  }
}
