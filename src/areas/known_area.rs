use std::sync::Arc;

use ratatui::layout::Position;

use super::*;

#[derive(Debug, Default)]
pub struct KnownArea {
  inner: Arc<Cell<Option<Rect>>>,
}
unsafe impl Send for KnownArea {}
unsafe impl Sync for KnownArea {}
impl KnownArea {
  pub fn replace(&self, inner: impl Into<Option<Rect>>) {
    self.inner.replace(inner.into());
  }

  pub fn intersects(&self, other: Position) -> bool {
    self.inner.get().is_some_and(|area| area.as_position() >= other)
  }
}
