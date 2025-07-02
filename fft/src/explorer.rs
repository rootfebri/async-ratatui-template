use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::prelude::StatefulWidget;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, HighlightSpacing, List, ListDirection, ListItem, ListState, Paragraph, Widget, Wrap};

use super::*;
use explorer_state::ExplorerState;

pub struct Explorer<'s> {
  state: &'s mut ExplorerState,
  selected_content: Option<ExplorerContent>,

  file_block: Option<Block<'s>>,
  input_block: Option<Block<'s>>,
}

impl<'s> Explorer<'s> {
  pub fn new(state: &'s mut ExplorerState) -> Self {
    let selected_content = state.selected_content_blocking();

    Self {
      state,
      selected_content,
      file_block: None,
      input_block: None,
    }
  }
  pub fn file_block(mut self, file_block: impl Into<Option<Block<'s>>>) -> Self {
    self.file_block = file_block.into();
    self
  }
  pub fn input_block(mut self, input_block: impl Into<Option<Block<'s>>>) -> Self {
    self.input_block = input_block.into();
    self
  }

  fn draw_filetree(&mut self) -> impl StatefulWidget<State = ListState> + 's {
    self
      .state
      .blocking_read_items()
      .iter()
      .map(|item| ListItem::new(item.apply_colors(self.state.input_state.as_str())))
      .collect::<List>()
      .block(self.file_block.take().unwrap_or_else(|| self.give_file_block()))
      .highlight_spacing(HighlightSpacing::Always)
      .highlight_style(Style::new().bg(Color::Rgb(50, 80, 70)).fg(Color::White).bold())
      .highlight_symbol("▶ ")
      .direction(ListDirection::BottomToTop)
  }

  fn give_file_block(&self) -> Block<'static> {
    let title_top_line = Line::raw(" File Tree ").centered().white();
    Block::bordered()
      .border_type(BorderType::Rounded)
      .fg(Color::Rgb(0, 255, 251))
      .title_top(title_top_line)
  }

  fn draw_input(&mut self) -> impl Widget {
    let input = self.state.input_state.as_str();
    let cursor_byte_pos = self.state.cursor.min(input.len()); // Clamp cursor to valid range
    let mut spans = Line::default();

    if input.is_empty() {
      // Show cursor in empty input
      spans.push_span(Span::raw(" ").dark_gray().on_white());
    } else if cursor_byte_pos >= input.len() {
      // Cursor at end of string
      spans.push_span(Span::raw(input));
      spans.push_span(Span::raw(" ").dark_gray().on_white());
    } else {
      // Make sure cursor is at a valid character boundary
      let adjusted_cursor = if input.is_char_boundary(cursor_byte_pos) {
        cursor_byte_pos
      } else {
        // Find the next valid character boundary
        (cursor_byte_pos..=input.len())
          .find(|&pos| input.is_char_boundary(pos))
          .unwrap_or(input.len())
      };

      // Render characters with cursor highlighting
      for (byte_pos, chr) in input.char_indices() {
        if byte_pos == adjusted_cursor {
          // Highlight the character at cursor position
          spans.push_span(Span::raw(chr.to_string()).dark_gray().on_white());
        } else {
          spans.push_span(Span::raw(chr.to_string()));
        }
      }
    }

    // Calculate scroll position based on cursor character position
    let cursor_char_pos = input[..cursor_byte_pos.min(input.len())].chars().count();
    let scroll_offset = cursor_char_pos.saturating_sub(10) as u16;

    Paragraph::new(spans)
      .wrap(Wrap { trim: true })
      .scroll((0, scroll_offset)) // Keep cursor visible with some padding
      .block(self.input_block.take().unwrap_or_else(|| self.give_input_block()))
  }

  fn give_input_block(&self) -> Block<'s> {
    let title_top_left = Line::raw(" Filter File/Dir(s) ").fg(Color::White).left_aligned();
    let title_top_right = Line::raw(format!(
      "{} / {}",
      self.state.blocking_read_items().len(),
      self.state.list.blocking_read().len()
    ))
    .fg(Color::DarkGray)
    .right_aligned();

    let title_bottom = Line::raw(self.state.watch_dir.borrow().to_string_lossy().into_owned());

    Block::bordered()
      .border_type(BorderType::Rounded)
      .fg(Color::Rgb(0, 255, 251))
      .title_top(title_top_left)
      .title_top(title_top_right)
      .title_bottom(title_bottom)
  }

  fn preview_block(&mut self) -> Block<'static> {
    let path_str = self
      .selected_content
      .as_ref()
      .map(ExplorerContent::filename)
      .map(String::from)
      .unwrap_or_default();
    let title_top = [Span::raw(" File Preview: ").white(), Span::raw(path_str).yellow()];

    Block::bordered()
      .border_type(BorderType::Rounded)
      .fg(Color::Rgb(0, 255, 251))
      .title_top(Line::from_iter(title_top).centered())
  }

  fn draw_preview(&mut self) -> impl Widget {
    let block = self.preview_block();
    if let Some(ref selected_content) = self.selected_content {
      selected_content.as_preview().block(block).left_aligned()
    } else {
      Paragraph::new("").block(block)
    }
  }
}

impl Widget for Explorer<'_> {
  fn render(mut self, area: Rect, buf: &mut Buffer) {
    let [left, content_area] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(Spacing::Space(1)).areas::<2>(area);
    let [file_area, input_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
      .spacing(Spacing::Space(1))
      .areas::<2>(left);

    Clear.render(file_area, buf);
    self.draw_filetree().render(file_area, buf, &mut self.state.list_state);
    Clear.render(input_area, buf);
    self.draw_input().render(input_area, buf);
    Clear.render(content_area, buf);
    self.draw_preview().render(content_area, buf);
  }
}
