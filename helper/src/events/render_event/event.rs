use ratatui::text::Span;
use strum::{Display, EnumIs};

#[derive(Clone, Debug, Default, Eq, PartialEq, EnumIs, Display)]
pub enum EventKind {
  #[default]
  Render,
  NoOps,
  #[strum(to_string = "{0}")]
  Error(Span<'static>),
  #[strum(to_string = "{0}")]
  Warn(Span<'static>),
  Handled,
  Canceled,
}
