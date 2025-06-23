use crossterm::event::Event;
use helper::{UnhandledEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Widget};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::Area;
use crate::areas::KnownArea;
use crate::ui::blk;

#[derive(Debug)]
pub struct Input {
  cursor: usize,
  value: String,
  label: String,
  placeholder: String,
  known_area: KnownArea,
}

impl Input {
  pub fn new(label: impl Into<Option<String>>, placeholder: impl Into<Option<String>>) -> Self {
    Self {
      cursor: 0,
      value: String::new(),
      label: label.into().unwrap_or(String::from("Input")).to_string(),
      placeholder: placeholder.into().unwrap_or(String::from("Start typing..")),
      known_area: Default::default(),
    }
  }
  pub fn handle_event(&mut self, event: &Event) -> Option<UnhandledEvent> {
    match event {
      Event::Key(keys!(Backspace, NONE, Press)) => {
        if self.cursor == 0 || self.value.is_empty() {
          return Some(UnhandledEvent::no_ops());
        }

        // Convert to char indices for proper Unicode handling
        let mut chars: Vec<char> = self.value.chars().collect();
        if self.cursor > 0 && self.cursor <= chars.len() {
          chars.remove(self.cursor - 1);
          self.value = chars.into_iter().collect();
          self.cursor -= 1;
        }

        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Delete, NONE, Press)) => {
        if self.value.is_empty() {
          return Some(UnhandledEvent::no_ops());
        }

        let mut chars: Vec<char> = self.value.chars().collect();
        if self.cursor < chars.len() {
          chars.remove(self.cursor);
          self.value = chars.into_iter().collect();
        }

        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Enter, NONE, Press)) => Some(UnhandledEvent::handled()),
      Event::Key(keys!(Esc, NONE, Press)) => Some(UnhandledEvent::canceled()),
      Event::Key(keys!(Left, NONE, Press)) => {
        self.cursor = self.cursor.saturating_sub(1);
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Right, NONE, Press)) => {
        let max_cursor = self.value.chars().count();
        if self.cursor < max_cursor {
          self.cursor += 1;
        }
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Home, NONE, Press)) => {
        self.cursor = 0;
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(End, NONE, Press)) => {
        self.cursor = self.value.chars().count();
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Char(chr), NONE, Press)) => {
        self.insert_char(*chr);
        Some(UnhandledEvent::render())
      }
      Event::Paste(content) => {
        self.paste(content);
        Some(UnhandledEvent::render())
      }
      _ => None,
    }
  }
  fn insert_char(&mut self, chr: char) {
    let mut chars: Vec<char> = self.value.chars().collect();
    chars.insert(self.cursor, chr);
    self.value = chars.into_iter().collect();
    self.cursor += 1;
  }

  fn paste(&mut self, content: impl AsRef<str>) {
    let content_str = content.as_ref();
    let mut chars: Vec<char> = self.value.chars().collect();

    for (i, chr) in content_str.chars().enumerate() {
      chars.insert(self.cursor + i, chr);
    }

    self.value = chars.into_iter().collect();
    self.cursor += content_str.chars().count();
  }

  pub fn value(self) -> String {
    self.value
  }
  fn draw_label(&self) -> Line {
    Line::raw(&self.label).left_aligned().fg(Color::Cyan)
  }

  fn draw_block(&self) -> Block {
    use ratatui::symbols::block::ONE_EIGHTH as I;

    let submit: Line = [
      Span::raw(" "),
      Span::raw("[ENTER]").fg(Color::Green),
      Span::raw(" Submit"),
      Span::raw(" "),
      Span::raw(I),
      Span::raw(" "),
      Span::raw("[ESC]").fg(Color::Red),
      Span::raw(" Cancel"),
      Span::raw(" "),
      Span::raw(I),
      Span::raw(" "),
      Span::raw("[⬅/➡/HOME/END]").fg(Color::Blue),
      Span::raw(" Navigate"),
      Span::raw(" "),
    ]
    .into_iter()
    .collect();

    blk()
      .title_top(self.draw_label())
      .title_bottom(submit.centered())
      .border_style(Color::White)
  }

  fn calculate_scroll_offset(&self, inner_width: usize) -> usize {
    if inner_width == 0 {
      return 0;
    }

    // Reserve space for cursor
    let available_width = inner_width.saturating_sub(1);

    if self.cursor < available_width {
      0
    } else {
      self.cursor.saturating_sub(available_width)
    }
  }
  #[inline(always)]
  fn cursor(&self) -> Span {
    Span::raw("│").fg(Color::Yellow).bg(Color::DarkGray)
  }
  fn draw_input(&self, area: Area, buf: &mut Buffer) {
    let block = self.draw_block();
    let inner = block.inner(area);

    // Calculate available width for text
    let inner_width = inner.width as usize;
    let scroll_offset = self.calculate_scroll_offset(inner_width);

    let spans = if self.value.is_empty() {
      // Show placeholder and cursor
      vec![Span::from(self.placeholder.as_str()).fg(Color::DarkGray).italic(), self.cursor()]
    } else {
      let chars: Vec<char> = self.value.chars().collect();
      let mut result_spans = Vec::new();

      // Apply horizontal scrolling
      let visible_start = scroll_offset;
      let visible_end = (scroll_offset + inner_width.saturating_sub(1)).min(chars.len());

      // Determine cursor position relative to visible area
      let cursor_in_view = self.cursor >= visible_start && self.cursor <= visible_end;
      let relative_cursor = self.cursor.saturating_sub(visible_start);

      if visible_start < chars.len() {
        // Add visible text before cursor
        if cursor_in_view && relative_cursor > 0 {
          let before_cursor: String = chars[visible_start..visible_start + relative_cursor].iter().collect();
          if !before_cursor.is_empty() {
            result_spans.push(Span::from(before_cursor).fg(Color::White));
          }
        }

        // Add cursor
        if cursor_in_view {
          result_spans.push(self.cursor());
        }

        // Add visible text after cursor
        if cursor_in_view && visible_start + relative_cursor < visible_end {
          let after_cursor: String = chars[visible_start + relative_cursor..visible_end].iter().collect();
          if !after_cursor.is_empty() {
            result_spans.push(Span::from(after_cursor).fg(Color::White));
          }
        } else if !cursor_in_view {
          // Cursor is not in view, just show the visible text
          let visible_text: String = chars[visible_start..visible_end].iter().collect();
          result_spans.push(Span::from(visible_text).fg(Color::White));
        }
      } else if cursor_in_view {
        // Only cursor is visible (at end of text)
        result_spans.push(self.cursor());
      }

      result_spans
    };

    Paragraph::new(Line::from_iter(spans)).block(block).render(area, buf);
  }
}

impl Widget for &Input {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    self.known_area.replace(area);
    self.draw_input(area, buf)
  }
}
