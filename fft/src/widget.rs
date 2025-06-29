use devicons::FileIcon;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, List, ListItem, Paragraph, StatefulWidget, Widget, Wrap};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
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
    self.file_block = input_block.into();
    self
  }

  fn draw_filetree(&mut self) -> impl Widget + 's {
    self
      .state
      .blocking_read_items()
      .into_iter()
      .map(|item| ListItem::new(item.apply_colors(self.state.input.as_str())))
      .collect::<List>()
      .block(self.file_block.take().unwrap_or_else(|| self.give_file_block()))
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
    let mut spans = Line::default();

    for (chrpos, (byte_pos, chr)) in input.char_indices().enumerate() {
      if chrpos == self.state.cursor {
        let highlighted = &input[byte_pos..byte_pos + chr.len_utf8()];
        spans.push_span(Span::raw(highlighted).dark_gray().on_white());
      } else {
        let normal = &input[byte_pos..byte_pos + chr.len_utf8()];
        spans.push_span(Span::raw(normal));
      }
    }

    Paragraph::new(spans)
      .wrap(Wrap { trim: true })
      .scroll((0, self.state.cursor as u16))
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
      selected_content.as_preview().block(block)
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
    let area = center_constraints(area, Constraint::Percentage(80), Constraint::Max(10));
    let [left, content_area] = Layout::horizontal([Constraint::Fill(1); 2]).spacing(Spacing::Space(1)).areas::<2>(area);
    let [file_area, input_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
      .spacing(Spacing::Space(1))
      .areas::<2>(left);

    // clear(content_area, buf);
    // clear(file_area, buf);
    // clear(input_area, buf);

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
    content: Arc<RwLock<Vec<u8>>>,
    state: Arc<RwLock<ExplorerContentState>>,
  },
}

impl ExplorerContent {
  pub fn icon(&self) -> FileIcon {
    FileIcon::from(self.as_path())
  }

  pub fn apply_colors(&self, input: &str) -> Line<'static> {
    if input.is_empty() {
      return self.as_cow().to_string().into();
    }

    let mut spans: Vec<Span> = vec![Span::raw(self.icon().icon.to_string()), Span::raw(" ")];

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

  pub async fn auto_load(&self) -> ! {
    let Self::File { ref state, ref content, .. } = *self else {
      loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(39)).await
      }
    };
    let mut state = state.write().await;
    'main: loop {
      match state.deref_mut() {
        ExplorerContentState::Start => match CharStream::new(self.as_path()).await {
          Ok(stream) => *state = ExplorerContentState::LinesBuffer(stream),
          Err(_) => *state = ExplorerContentState::Done,
        },
        ExplorerContentState::LinesBuffer(stream) => loop {
          if let Ok(Some(c)) = stream.next_char().await {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            content.write().await.extend_from_slice(encoded.as_bytes());
          } else {
            *state = ExplorerContentState::Done;
            continue 'main;
          }
        },
        ExplorerContentState::Done => break,
      }
    }

    drop(state);

    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(39)).await
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

  pub fn content(&self) -> Cow<'_, str> {
    match *self {
      ExplorerContent::Dir { .. } => String::from_utf8_lossy(self.as_path().as_os_str().as_encoded_bytes()),
      ExplorerContent::File { ref content, .. } => String::from_utf8_lossy(content.blocking_read().as_slice()).into_owned().into(),
    }
  }

  pub fn as_item(&self) -> ListItem {
    let path: Cow<'_, str> = match *self {
      ExplorerContent::Dir { ref path, .. } | ExplorerContent::File { ref path, .. } => String::from_utf8_lossy(path.as_os_str().as_encoded_bytes()),
    };

    ListItem::new(path)
  }

  pub fn as_preview(&self) -> Paragraph {
    let content = Line::from(self.content()).centered().fg(Color::LightBlue);
    Paragraph::new(content)
  }

  pub fn as_static_list_item(&self) -> ListItem<'static> {
    let path_str = self.as_cow().into_owned();
    let file_icon = FileIcon::from(self.as_path());
    let line = [Span::raw(file_icon.icon.to_string()), Span::raw(" "), Span::raw(path_str)]
      .into_iter()
      .collect::<Line>();

    ListItem::new(line)
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
  LinesBuffer(CharStream),
  Done,
}

impl StatefulWidget for ExplorerContent {
  type State = (u16, u16);
  fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    let border = Block::bordered()
      .border_type(BorderType::Rounded)
      .fg(Color::Rgb(0, 255, 251))
      .title_top(" File Content Preview ");

    Paragraph::new(self.content())
      .block(border)
      .wrap(Wrap { trim: false })
      .scroll(*state)
      .render(area, buf);
  }
}
