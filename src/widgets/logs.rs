use std::collections::VecDeque;
use std::sync::Arc;

use crate::app::ScrollState;
use crate::areas::KnownArea;
use crate::mouse_area;
use crate::ui::{blk, clear};
use crate::widgets::Log;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{Stylize, Text, Widget};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Logs {
  items: Arc<RwLock<VecDeque<Log>>>,
  state: Arc<RwLock<ScrollState>>,
  known_area: KnownArea,
}

impl Logs {
  pub async fn handle_key(&self, key: KeyEvent) -> Option<RenderEvent> {
    let items_count = self.items.read().await.len();
    let visible_height = self.known_area.area().height as usize;
    let max_scroll = if items_count > visible_height {
      (items_count - visible_height) as u16
    } else {
      0
    };

    match key {
      keys!(Up, NONE, Press) => {
        if items_count > visible_height {
          self.state.write().await.scroll_up(max_scroll);
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(Down, NONE, Press) => {
        if items_count > visible_height {
          self.state.write().await.scroll_down(max_scroll);
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(PageUp, NONE, Press) => {
        if items_count > visible_height {
          let scroll_amount = (visible_height / 2).max(1) as u16;
          self.state.write().await.scroll_up_by(scroll_amount, max_scroll);
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(PageDown, NONE, Press) => {
        if items_count > visible_height {
          let scroll_amount = (visible_height / 2).max(1) as u16;
          self.state.write().await.scroll_down_by(scroll_amount, max_scroll);
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(Home, NONE, Press) => {
        self.state.write().await.scroll_to_top();
        Some(RenderEvent::render())
      }
      keys!(Char(' '), CONTROL, Press) | keys!(End, NONE, Press) => {
        if items_count > visible_height {
          self.state.write().await.scroll_to_bottom(max_scroll);
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      _ => None,
    }
  }

  pub async fn handle_mouse(&self, mouse_event: MouseEvent) -> Option<RenderEvent> {
    use MouseEventKind::{ScrollDown, ScrollUp};

    let mouse_area = mouse_area(&mouse_event).as_position();

    if !self.intersects(mouse_area) {
      return None;
    }

    let items_count = self.items.read().await.len();
    let visible_height = self.known_area.area().height as usize;
    let max_scroll = if items_count > visible_height {
      (items_count - visible_height) as u16
    } else {
      0
    };

    match mouse_event.kind {
      ScrollUp => {
        if items_count > visible_height {
          self.state.write().await.scroll_up_by(3, max_scroll);
        }
      }
      ScrollDown => {
        if items_count > visible_height {
          self.state.write().await.scroll_down_by(3, max_scroll);
        }
      }
      _ => return None,
    }

    Some(RenderEvent::render())
  }

  pub fn intersects(&self, position: Position) -> bool {
    self.known_area.intersects(position)
  }

  pub async fn info(&self, data: impl Into<Arc<str>>) {
    self.add(Log::info(data)).await;
  }

  pub async fn add(&self, log: Log) {
    let mut items = self.items.write().await;
    let visible_height = self.known_area.area().height as usize;

    items.push_back(log);
    if items.len() > 5000 {
      items.pop_front();
    }

    let items_count = items.len();
    drop(items);

    // Auto-scroll if enabled and we have more items than visible
    if items_count > visible_height {
      let mut state = self.state.write().await;
      state.auto_scroll(items_count, visible_height);
    }
  }

  fn hotkey_labels(&self) -> Line {
    use ratatui::symbols::block::ONE_EIGHTH as I;

    let hotkeys: Line = [
      Span::raw(" "),
      Span::raw("[↑/↓]").fg(Color::Green),
      Span::raw(" Navigate"),
      Span::raw(" "),
      Span::raw(I),
      Span::raw(" "),
      Span::raw("[PgUp/PgDn]").fg(Color::Blue),
      Span::raw(" Fast Scroll"),
      Span::raw(" "),
      Span::raw(I),
      Span::raw(" "),
      Span::raw("[Home/End]").fg(Color::Yellow),
      Span::raw(" Jump"),
      Span::raw(" "),
    ]
    .into_iter()
    .collect();

    hotkeys
  }
  fn draw_logs_counter(&self) -> Line {
    let items_count = self.items.blocking_read().len();
    let ScrollState { vertical, auto_scroll, .. } = *self.state.blocking_read();
    let visible_height = self.known_area.area().height as usize;
    let current_line = if items_count > visible_height { vertical + 1 } else { 1 };
    let auto_indicator = if auto_scroll { "🔄" } else { "🔒" };

    [
      Span::raw(ratatui::symbols::line::VERTICAL_LEFT),
      Span::raw(format!(" {auto_indicator} ")),
      Span::styled(current_line.to_string(), Color::Yellow),
      Span::raw("/"),
      Span::styled(items_count.to_string(), Color::White),
      Span::raw(" "),
      Span::raw(ratatui::symbols::line::VERTICAL_RIGHT),
    ]
    .into_iter()
    .collect::<Line>()
  }
}

impl Widget for &Logs {
  fn render(self, area: Rect, buf: &mut Buffer) {
    self.known_area.replace(area);

    let title = Line::raw(" 📈 Activities ").left_aligned();
    let logs_counter = self.draw_logs_counter().right_aligned();

    let block = blk()
      .title_top(title)
      .title_bottom(logs_counter)
      .title_bottom(self.hotkey_labels().centered());

    let locked_items = self.items.blocking_read();
    let lines: Text = locked_items.iter().map(Line::from).collect();
    let widget = Paragraph::new(lines)
      .block(block)
      .wrap(Wrap { trim: false })
      .scroll(self.state.blocking_read().as_tuple());

    clear(area, buf);
    widget.render(area, buf);
  }
}

impl Clone for Logs {
  fn clone(&self) -> Self {
    Self {
      items: Arc::clone(&self.items),
      state: Arc::clone(&self.state),
      known_area: self.known_area.clone(),
    }
  }
}
