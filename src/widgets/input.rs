use crate::areas::KnownArea;
use crate::ui::blk;
use crossterm::event::Event;
use fft::InputState;
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Widget};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Input {
  state: RefCell<InputState>,
  label: String,
  placeholder: String,
  known_area: KnownArea,
}

impl Input {
  pub fn new(label: impl Into<Option<String>>, placeholder: impl Into<Option<String>>) -> Self {
    Self {
      state: RefCell::new(InputState::new("")),
      label: label.into().unwrap_or(String::from("Input")).to_string(),
      placeholder: placeholder.into().unwrap_or(String::from("Start typing..")),
      known_area: Default::default(),
    }
  }

  pub fn handle_event(&mut self, event: &Event) -> Option<RenderEvent> {
    match event {
      Event::Key(keys!(Enter, NONE, Press)) => return Some(RenderEvent::handled()),
      Event::Key(keys!(Esc, NONE, Press)) => return Some(RenderEvent::canceled()),

      Event::Key(keys!(Backspace, NONE, Press)) => self.state.get_mut().backspace(),
      Event::Key(keys!(Backspace, CONTROL, Press)) => self.state.get_mut().ctrl_backspace(),
      Event::Key(keys!(Delete, NONE, Press)) => self.state.get_mut().delete(),
      Event::Key(keys!(Delete, CONTROL, Press)) => self.state.get_mut().ctrl_delete(),

      Event::Key(keys!(Left, NONE, Press)) => self.state.get_mut().left(),
      Event::Key(keys!(Left, CONTROL, Press)) => self.state.get_mut().move_left_word(),
      Event::Key(keys!(Right, NONE, Press)) => self.state.get_mut().right(),
      Event::Key(keys!(Right, CONTROL, Press)) => self.state.get_mut().move_right_word(),

      Event::Key(keys!(Home, NONE, Press)) => self.state.get_mut().set_cursor(0),
      Event::Key(keys!(End, NONE, Press)) => {
        let len = self.state.borrow().len();
        self.state.get_mut().set_cursor(len)
      }
      Event::Key(keys!(Char(chr), NONE, Press)) => self.state.get_mut().push(*chr),
      Event::Paste(content) => self.state.get_mut().push_str(content),

      _ => return None,
    }

    Some(RenderEvent::render())
  }

  pub fn value(self) -> String {
    self.state.into_inner().into()
  }

  fn draw_label(&self) -> Line {
    Line::raw(&self.label).left_aligned().fg(Color::Cyan)
  }

  fn draw_block(&self) -> Block {
    use ratatui::symbols::block::ONE_EIGHTH as I;

    let hotkeys: Line = [
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
      .title_bottom(hotkeys.centered())
      .border_style(Color::White)
  }

  fn draw_input(&self, area: Rect, buf: &mut Buffer) {
    let block = self.draw_block();
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
      return;
    }

    let state = self.state.borrow();
    let text = state.as_str();
    let cursor_pos = state.cursor();

    // Determine what to display
    let (display_text, cursor_style, text_style) = if text.is_empty() {
      // Show placeholder when empty
      (self.placeholder.as_str(), Color::DarkGray, Color::DarkGray)
    } else {
      // Show actual text
      (text, Color::Yellow, Color::White)
    };

    // Calculate visible text area (leave space for cursor)
    let text_area = Rect {
      x: inner.x + 1,
      y: inner.y + (inner.height / 2),
      width: inner.width.saturating_sub(2),
      height: 1,
    };

    if text_area.width == 0 {
      return;
    }

    // Handle text scrolling if it's too long
    let max_visible_chars = text_area.width as usize;
    let (visible_text, visible_cursor_pos) = if display_text.chars().count() > max_visible_chars {
      let cursor_char_pos = text[..cursor_pos.min(text.len())].chars().count();

      // Calculate scroll offset to keep cursor visible
      let scroll_offset = if cursor_char_pos >= max_visible_chars {
        cursor_char_pos.saturating_sub(max_visible_chars - 1)
      } else {
        0
      };

      let visible: String = display_text
        .chars()
        .skip(scroll_offset)
        .take(max_visible_chars)
        .collect();

      let visible_cursor = cursor_char_pos.saturating_sub(scroll_offset);
      (visible, visible_cursor)
    } else {
      let cursor_char_pos = if text.is_empty() { 0 } else { text[..cursor_pos.min(text.len())].chars().count() };
      (display_text.to_string(), cursor_char_pos)
    };

    // Render the text
    let text_line = Line::raw(&visible_text).style(text_style);
    text_line.render(text_area, buf);

    // Render cursor only if we're showing actual text (not placeholder)
    if !text.is_empty() || visible_cursor_pos == 0 {
      let cursor_x = text_area.x + visible_cursor_pos as u16;
      if cursor_x < text_area.x + text_area.width {
        // Render cursor as a highlighted character or block
        let cursor_char = if visible_cursor_pos < visible_text.chars().count() {
          visible_text.chars().nth(visible_cursor_pos).unwrap_or(' ')
        } else {
          ' '
        };

        let cursor_span = Span::raw(cursor_char.to_string())
          .bg(cursor_style)
          .fg(Color::Black);

        let cursor_line = Line::from(cursor_span);
        let cursor_area = Rect {
          x: cursor_x,
          y: text_area.y,
          width: 1,
          height: 1,
        };
        cursor_line.render(cursor_area, buf);
      }
    }
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
