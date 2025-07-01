use ratatui::prelude::*;
use std::borrow::Cow;

pub use pulse_fps::*;
pub use pulse_level::*;
pub use pulse_state::*;

mod pulse_fps;
mod pulse_level;
mod pulse_state;

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct Pulse<'s> {
  content: Cow<'s, str>,
  level: PulseLevel,
}

impl<'s> Pulse<'s> {
  pub fn new<C: Into<Cow<'s, str>>>(content: C) -> Self {
    Self {
      content: content.into(),
      level: Default::default(),
    }
  }

  pub fn level(mut self, level: PulseLevel) -> Self {
    self.level = level;
    self
  }
}

impl StatefulWidget for &Pulse<'_> {
  type State = PulseState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    Span::styled(self.content.as_ref(), state.color(self.level)).render(area, buf);
  }
}
impl StatefulWidget for Pulse<'_> {
  type State = PulseState;

  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    Span::styled(self.content.as_ref(), state.color(self.level)).render(area, buf);
  }
}
