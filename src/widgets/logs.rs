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
use ratatui::widgets::Paragraph;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Logs {
  items: Arc<RwLock<VecDeque<Log>>>,
  state: Arc<RwLock<ScrollState>>,
  known_area: KnownArea,
}

impl Logs {
  pub async fn handle_key(&self, key: KeyEvent) -> Option<RenderEvent> {
    match key {
      keys!(Up, NONE, Press) => {
        if self.items.read().await.len() > self.known_area.area().height as usize {
          self.state.write().await.scroll_up();
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(Down, NONE, Press) => {
        if self.items.read().await.len() > self.known_area.area().height as usize {
          self.state.write().await.scroll_down();
          Some(RenderEvent::render())
        } else {
          None
        }
      }
      keys!(PageUp, NONE, Press) => {
        self.state.write().await.lock();

        let height = self.known_area.area().height as usize;
        let items_count = self.items.read().await.len();

        if items_count > height {
          if items_count - 10 > height {
            self.state.write().await.scroll_up_by(10);
          } else {
            self.state.write().await.scroll_up_by((items_count - 10) as u16);
          }
          Some(RenderEvent::render())
        } else {
          Some(RenderEvent::no_ops())
        }
      }
      keys!(PageDown, NONE, Press) => {
        self.state.write().await.lock();

        let height = self.known_area.area().height as usize;
        let items_count = self.items.read().await.len();

        if items_count > height {
          if items_count - 10 > height {
            self.state.write().await.scroll_up_by(10);
          } else {
            self.state.write().await.scroll_up_by((items_count - 10) as u16);
          }
          Some(RenderEvent::render())
        } else {
          Some(RenderEvent::no_ops())
        }
      }
      keys!(Home, NONE, Press) => {
        self.state.write().await.scroll_to_top();
        Some(RenderEvent::render())
      }
      keys!(Char(' '), CONTROL, Press) | keys!(End, NONE, Press) => {
        {
          let mut state = self.state.write().await;
          state.unlock();
          state.scroll_down_by(self.items.read().await.len() as u16);
        }

        Some(RenderEvent::render())
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

    match mouse_event.kind {
      ScrollUp => self.state.write().await.scroll_up_by(1),
      ScrollDown => self.state.write().await.scroll_down_by(1),
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
    {
      let mut state = self.state.write().await;
      if items.len() > state.vertical as usize {
        state.auto_scroll();
      }
    }

    items.push_back(log);
    if items.len() > 5000 {
      items.pop_front();
    }
    drop(items);
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
    let ScrollState { vertical, horizontal, .. } = *self.state.blocking_read();

    [
      Span::raw(ratatui::symbols::line::VERTICAL_LEFT),
      Span::raw(format!("[{vertical}/{horizontal}]")),
      Span::raw(" "),
      Span::styled(vertical.to_string(), Color::Gray),
      Span::raw("/"),
      Span::styled(items_count.to_string(), Color::White),
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
    let widget = Paragraph::new(lines).block(block).scroll(self.state.blocking_read().as_tuple());

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
