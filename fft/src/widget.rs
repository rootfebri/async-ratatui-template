use devicons::FileIcon;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, HighlightSpacing, List, ListItem, Paragraph, Widget, Wrap};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::DerefMut;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::sync::RwLock;

use super::*;
use state::ExplorerState;

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

  fn draw_filetree(&mut self) -> impl Widget + 's {
    self
      .state
      .blocking_read_items()
      .iter()
      .map(|item| ListItem::new(item.apply_colors(self.state.input.as_str())))
      .collect::<List>()
      .block(self.file_block.take().unwrap_or_else(|| self.give_file_block()))
      .highlight_spacing(HighlightSpacing::Always)
      .highlight_style(Style::new().bg(Color::Rgb(131, 164, 150)))
      .highlight_symbol(">")
  }

  fn give_file_block(&self) -> Block<'static> {
    let title_top_line = Line::raw(" File Tree ").centered().white();
    Block::bordered()
      .border_type(BorderType::Rounded)
      .fg(Color::Rgb(0, 255, 251))
      .title_top(title_top_line)
  }

  fn draw_input(&mut self) -> impl Widget {
    let input = self.state.input.as_str();
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

  fn give_input_block(&self) -> Block {
    let title_top_left = Line::raw(" Filter File/Dir(s) ").fg(Color::White).left_aligned();
    let title_top_right = Line::raw(format!(
      "{} / {}",
      self.state.blocking_read_items().len(),
      self.state.list.blocking_read().len()
    ))
    .fg(Color::DarkGray)
    .right_aligned();

    let title_bottom = Line::raw(self.state.cwd.to_string_lossy());

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
      .map(|selected| selected.as_cow().to_string())
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
      selected_content.as_preview().block(block).wrap(Wrap { trim: false }).left_aligned()
    } else {
      Paragraph::new("").block(block)
    }
  }
}

fn center_constraints(area: Rect, w: Constraint, h: Constraint) -> Rect {
  let [_, w, _] = Layout::horizontal([Constraint::Fill(1), w, Constraint::Fill(1)]).areas(area);
  Layout::vertical([Constraint::Fill(1), h, Constraint::Fill(1)]).split(w)[1]
}

impl Widget for Explorer<'_> {
  fn render(mut self, area: Rect, buf: &mut Buffer) {
    let area = center_constraints(area, Constraint::Percentage(80), Constraint::Percentage(90));
    let [left, content_area] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(Spacing::Space(1)).areas::<2>(area);
    let [file_area, input_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
      .spacing(Spacing::Space(1))
      .areas::<2>(left);

    Clear.render(content_area, buf);
    Clear.render(file_area, buf);
    Clear.render(input_area, buf);

    self.draw_filetree().render(file_area, buf);
    self.draw_input().render(input_area, buf);
    self.draw_preview().render(content_area, buf);
  }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExplorerContent {
  Dir {
    path: Arc<Path>,
  },
  File {
    path: Arc<Path>,
    content: Arc<RwLock<String>>,
    state: Arc<RwLock<ExplorerContentState>>,
  },
}

impl ExplorerContent {
  pub fn icon(&self) -> Span<'static> {
    let fileicon = FileIcon::from(self.as_path());
    let color = Color::from_str(fileicon.color).unwrap_or(Color::Reset);
    let icon = fileicon.icon.to_string();

    Span::raw(icon).fg(color)
  }

  pub fn apply_colors(&self, input: &str) -> Line<'static> {
    let icon = self.icon();
    if input.is_empty() {
      let mut line = Line::default();
      line.push_span(icon);
      line.push_span(Span::raw(" "));
      line.push_span(Span::raw(self.as_cow().to_string()).white());

      return line;
    };

    let mut spans: Vec<Span> = vec![icon, Span::raw(" ")];

    let path_str = self.as_cow();
    let input_chars: Vec<_> = input.chars().collect();

    for chr in path_str.chars() {
      let span = Span::raw(chr.to_string());

      if input_chars.contains(&chr) {
        spans.push(span.fg(Color::Rgb(36, 132, 96)));
      } else {
        spans.push(span.fg(Color::White));
      }
    }

    spans.into()
  }

  pub async fn open_lines_buffered(&self) -> Option<Lines<BufReader<File>>> {
    let ExplorerContent::File { ref path, .. } = *self else { return None };
    let file = File::open(path).await.ok()?;
    Some(BufReader::new(file).lines())
  }

  async fn never() -> ! {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(39)).await
    }
  }

  pub async fn auto_load(&self) {
    let Self::File { ref state, ref content, .. } = *self else {
      Self::never().await
    };

    'main: loop {
      let mut state = state.write().await;
      match state.deref_mut() {
        ExplorerContentState::Start => {
          if let Some(stream) = self.open_lines_buffered().await {
            *state = ExplorerContentState::LinesBuffer(Box::from(stream));
          } else {
            *state = ExplorerContentState::Done;
          }
        }
        ExplorerContentState::LinesBuffer(stream) => {
          if let Ok(Some(ref str)) = stream.next_line().await {
            content.write().await.push_str(str);
            content.write().await.push('\n');
            continue 'main;
          } else {
            *state = ExplorerContentState::Done;
            continue 'main;
          }
        }
        ExplorerContentState::Done => break,
      }
    }
  }

  pub fn is_file(&self) -> bool {
    matches!(self, Self::File { .. })
  }

  pub fn as_path(&self) -> &Path {
    match *self {
      ExplorerContent::Dir { ref path, .. } | ExplorerContent::File { ref path, .. } => path.as_ref(),
    }
  }

  pub fn content(&self) -> String {
    std::thread::sleep(std::time::Duration::from_millis(1));

    match *self {
      ExplorerContent::Dir { .. } => String::new(),
      ExplorerContent::File { ref content, .. } => content.blocking_read().clone(),
    }
  }

  pub fn as_preview(&self) -> Paragraph {
    let content = Line::from(self.content()).fg(Color::White);
    Paragraph::new(content)
  }

  pub fn as_cow(&self) -> Cow<'_, str> {
    self.as_path().to_string_lossy()
  }
}

impl PartialEq for ExplorerContent {
  fn eq(&self, other: &Self) -> bool {
    self.as_path() == other.as_path()
  }
}

impl PartialOrd for ExplorerContent {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.as_path().cmp(other.as_path()))
  }
}

impl Ord for ExplorerContent {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}

impl Eq for ExplorerContent {}

#[derive(Debug, Default)]
pub enum ExplorerContentState {
  #[default]
  Start,
  LinesBuffer(Box<Lines<BufReader<File>>>),
  Done,
}
