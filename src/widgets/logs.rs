use std::collections::VecDeque;
use std::ops::DerefMut;
use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use helper::{UnhandledEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::text::Line;
use ratatui::widgets::{List, ListDirection, ListState};
use tokio::sync::RwLock;

use crate::areas::KnownArea;
use crate::mouse_area;
use crate::ui::{blk, clear};
use crate::widgets::Log;

#[derive(Debug, Default)]
pub struct Logs {
  items: Arc<RwLock<VecDeque<Log>>>,
  state: Arc<RwLock<ListState>>,
  known_area: KnownArea,
}

impl Logs {
  pub async fn handle_key(&self, key: KeyEvent) -> Option<UnhandledEvent> {
    match key {
      keys!(Up, NONE, Press) => {
        self.state.write().await.scroll_up_by(1);
        Some(UnhandledEvent::render())
      }
      keys!(Down, NONE, Press) => {
        self.state.write().await.scroll_down_by(1);
        Some(UnhandledEvent::render())
      }
      keys!(PageUp, NONE, Press) => {
        self.state.write().await.scroll_up_by(10);
        Some(UnhandledEvent::render())
      }
      keys!(PageDown, NONE, Press) => {
        self.state.write().await.scroll_down_by(10);
        Some(UnhandledEvent::render())
      }
      keys!(Home, NONE, Press) => {
        let items_count = self.items.read().await.len();
        if items_count > 0 {
          self.state.write().await.select(Some(items_count - 1));
        }
        Some(UnhandledEvent::render())
      }
      keys!(End, NONE, Press) => {
        self.state.write().await.select(Some(0));
        Some(UnhandledEvent::render())
      }
      _ => None,
    }
  }

  pub async fn handle_mouse(&self, mouse_event: MouseEvent) -> Option<UnhandledEvent> {
    use MouseEventKind::{ScrollDown, ScrollUp};

    let mouse_area = mouse_area(&mouse_event).as_position();

    if !self.intersects(mouse_area) {
      return None;
    }

    match mouse_event.kind {
      ScrollUp => self.state.write().await.scroll_up_by(1),
      ScrollDown => self.state.write().await.scroll_down_by(1),
      _ => return None,
    }

    Some(UnhandledEvent::render())
  }

  pub fn intersects(&self, position: Position) -> bool {
    self.known_area.intersects(position)
  }

  pub async fn add(&self, log: Log) {
    let mut items = self.items.write().await;
    items.push_back(log);
    if items.len() > 5000 {
      items.pop_front();
    }
  }
}

impl Clone for Logs {
  fn clone(&self) -> Self {
    Self {
      items: Arc::clone(&self.items),
      state: Default::default(),
      known_area: Default::default(),
    }
  }
}

impl Widget for &Logs {
  fn render(self, area: Rect, buf: &mut Buffer) {
    self.known_area.replace(area);
    clear(area, buf);

    let title = Line::raw(" 📈Activities ").left_aligned();
    let block = blk().title_top(title);
    let locked_items = self.items.blocking_read();
    let items = locked_items.iter().map(Log::as_list_item).rev().collect::<Vec<_>>();
    let list = List::default().items(items).block(block).direction(ListDirection::BottomToTop);

    {
      let mut state = self.state.blocking_write();
      StatefulWidget::render(list, area, buf, state.deref_mut())
    }
  }
}
