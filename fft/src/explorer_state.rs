use super::*;
use crossterm::event::KeyEvent;
use helper::{RenderEvent, keys};
use ratatui::widgets::ListState;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, watch};
use tokio::task::JoinHandle;
use tokio::time::sleep;

impl Default for ExplorerState {
  fn default() -> Self {
    Self::new(None)
  }
}

pub struct ExplorerState {
  #[allow(dead_code)]
  dir_scanner: JoinHandle<()>,
  pub(crate) watch_dir: watch::Sender<Arc<Path>>,

  pub(crate) cursor: usize, // byte position in input string
  pub(crate) input: String,

  pub(crate) list: Arc<RwLock<Vec<ExplorerContent>>>,

  pub(crate) list_state: ListState,
}

impl ExplorerState {
  fn new(entry: impl Into<Option<PathBuf>>) -> Self {
    let canonicalized = Self::realpath_blocking(entry.into().unwrap_or_else(|| PathBuf::from("./")));
    let (watch_tx, watch_rx) = watch::channel(Arc::from(canonicalized));
    let list: Arc<RwLock<Vec<ExplorerContent>>> = Default::default();

    Self {
      dir_scanner: tokio::spawn(parent_content_scanner(watch_rx, list.clone())),
      watch_dir: watch_tx,
      cursor: 0,
      input: String::new(),
      list,
      list_state: ListState::default(),
    }
  }

  async fn update_watched_cwd(&mut self, path: impl Into<PathBuf>) -> Option<RenderEvent> {
    let path = path.into();
    if !path.is_dir() {
      return None;
    }

    let canonicalized = Self::realpath(path).await;

    self.list.write().await.clear();
    self.watch_dir.send_modify(|dir| *dir = Arc::from(canonicalized));

    // Give the scanner a moment to start processing
    sleep(Duration::from_millis(16)).await;
    // Sync the new list contents and ensure selection
    self.ensure_selection().await;

    Some(RenderEvent::render())
  }

  async fn selected_content(&self) -> Option<ExplorerContent> {
    self.read_items().await.get(self.list_state.selected()?).cloned()
  }

  pub fn handle_paste(&mut self, content: &str) -> RenderEvent {
    // Ensure cursor is at a valid position
    self.normalize_cursor();

    // Insert at current byte position
    if self.input.is_char_boundary(self.cursor) {
      self.input.insert_str(self.cursor, content);
      self.cursor += content.len();

      // Normalize cursor after insertion
      self.normalize_cursor();
    }

    RenderEvent::render()
  }

  pub async fn handle_key(&mut self, event: KeyEvent) -> Option<RenderEvent> {
    self.ensure_selection().await;

    match event {
      keys!(Up, NONE, Press) => self.list_state.select_next(),
      keys!(Down, NONE, Press) => self.list_state.select_previous(),
      keys!(Left, NONE, Press) => self.move_cursor_left(),
      keys!(Right, NONE, Press) => self.move_cursor_right(),

      keys!(Char(chr), NONE, Press) => {
        self.insert_char(chr);
        self.clamp_selection_async().await;
      }
      keys!(Backspace, NONE, Press) => {
        self.delete_char_before_cursor();
        self.clamp_selection_async().await;
      }
      keys!(Delete, NONE, Press) => {
        self.delete_char_at_cursor();
        self.clamp_selection_async().await;
      }
      keys!(Backspace, CONTROL, Press) => {
        self.remove_word_backwards();
        self.clamp_selection_async().await;
      }
      keys!(Delete, CONTROL, Press) => {
        self.remove_word_forwards();
        self.clamp_selection_async().await;
      }
      keys!(Home, NONE, Press) => self.move_cursor_home(),
      keys!(End, NONE, Press) => self.move_cursor_end(),
      keys!(Left, CONTROL, Press) => self.move_cursor_word_left(),
      keys!(Right, CONTROL, Press) => self.move_cursor_word_right(),

      keys!(Esc, NONE, Press) => return Some(RenderEvent::canceled()),
      keys!(Enter, NONE, Press) => return Some(RenderEvent::handled()),
      keys!(Left, ALT, Press) => {
        let mut cwd = self.watch_dir.borrow().to_path_buf();
        return if cwd.pop() { self.update_watched_cwd(cwd).await } else { None };
      }
      keys!(Right, ALT, Press) => {
        let content = self.selected_content().await.and_then(|c| if c.is_dir() { Some(c) } else { None })?;
        return self.update_watched_cwd(content.as_path().to_path_buf()).await;
      }

      keys!(Char('a'), CONTROL, Press) => self.select_all(),
      keys!(Char('u'), CONTROL, Press) => {
        self.clear_input();
        self.clamp_selection_async().await;
      }

      _ => return None,
    }

    // Always normalize cursor after text operations
    self.normalize_cursor();
    Some(RenderEvent::render())
  }

  pub async fn get(self) -> Option<PathBuf> {
    self
      .read_items()
      .await
      .get(self.list_state.selected()?)
      .map(|content| content.as_path().to_path_buf())
  }

  pub(crate) fn selected_content_blocking(&self) -> Option<ExplorerContent> {
    self.blocking_read_items().get(self.list_state.selected()?).cloned()
  }

  pub(crate) fn blocking_read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.blocking_read();
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.filename().fuzzy_contains(self.input.as_str()))
      .map(ExplorerContent::clone)
      .collect::<Vec<_>>();
    drop(list);

    items.sort_unstable_by(|a, b| match (a.is_file(), b.is_file()) {
      (true, true) => Equal,
      (true, false) | (false, true) => Greater,
      (false, false) => Less,
    });

    if !self.input.is_empty() {
      items.sort_unstable_by(|a, b| {
        let a_score = a.filename().fuzzy_score(self.input.as_str());
        let b_score = b.filename().fuzzy_score(self.input.as_str());
        a_score.cmp(&b_score)
      });
    }

    items
  }

  pub(crate) async fn read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.read().await;
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.filename().fuzzy_contains(self.input.as_str()))
      .map(ExplorerContent::clone)
      .collect::<Vec<_>>();

    drop(list);

    items.sort_unstable_by(|a, b| match (a.is_file(), b.is_file()) {
      (true, true) => Equal,
      (true, false) | (false, true) => Greater,
      (false, false) => Less,
    });

    items
  }

  fn remove_word_backwards(&mut self) {
    if self.cursor == 0 {
      return;
    }

    // Start from the current cursor position in characters
    let chars: Vec<char> = self.input.chars().collect();
    let original_char_pos = self.input[..self.cursor].chars().count();
    let mut char_pos = original_char_pos;

    // If we're at the end or on whitespace, move back to find non-whitespace
    if char_pos > 0 {
      char_pos -= 1;

      // Skip any trailing whitespace
      while char_pos > 0 && chars[char_pos].is_whitespace() {
        char_pos -= 1;
      }

      // Move backwards to find the start of the word
      while char_pos > 0 && !chars[char_pos].is_whitespace() {
        char_pos -= 1;
      }

      // If we stopped on whitespace, move forward to the start of the word
      if char_pos > 0 && chars[char_pos].is_whitespace() {
        char_pos += 1;
      }
    }

    if char_pos < original_char_pos {
      // Convert character positions to byte positions
      let start_byte = self.input.char_indices().nth(char_pos).map(|(pos, _)| pos).unwrap_or(0);
      let end_byte = self.cursor;

      self.input.drain(start_byte..end_byte);
      self.cursor = start_byte;
    }
  }

  fn remove_word_forwards(&mut self) {
    if self.cursor >= self.input.len() {
      return;
    }

    let chars: Vec<char> = self.input.chars().collect();
    let original_char_pos = self.input[..self.cursor].chars().count();
    let mut char_pos = original_char_pos;

    // Skip any leading whitespace at cursor
    while char_pos < chars.len() && chars[char_pos].is_whitespace() {
      char_pos += 1;
    }

    // Find the end of the word
    while char_pos < chars.len() && !chars[char_pos].is_whitespace() {
      char_pos += 1;
    }

    if char_pos > original_char_pos {
      // Convert character positions to byte positions
      let start_byte = self.cursor;
      let end_byte = self.input.char_indices().nth(char_pos).map(|(pos, _)| pos).unwrap_or(self.input.len());

      self.input.drain(start_byte..end_byte);
      // cursor stays at same position
    }
  }

  /// Move cursor to the previous character boundary
  fn move_cursor_left(&mut self) {
    if self.cursor == 0 {
      return;
    }

    // Find the start of the previous character
    let mut new_cursor = self.cursor - 1;
    while new_cursor > 0 && !self.input.is_char_boundary(new_cursor) {
      new_cursor -= 1;
    }
    self.cursor = new_cursor;
  }

  /// Move cursor to the next character boundary
  fn move_cursor_right(&mut self) {
    if self.cursor >= self.input.len() {
      return;
    }

    // Find the start of the next character
    let mut new_cursor = self.cursor + 1;
    while new_cursor < self.input.len() && !self.input.is_char_boundary(new_cursor) {
      new_cursor += 1;
    }
    self.cursor = new_cursor;
  }

  /// Move cursor to start of input
  fn move_cursor_home(&mut self) {
    self.cursor = 0;
  }

  /// Move cursor to end of input
  fn move_cursor_end(&mut self) {
    self.cursor = self.input.len();
  }

  /// Move cursor by word boundaries
  fn move_cursor_word_left(&mut self) {
    if self.cursor == 0 {
      return;
    }

    let chars: Vec<char> = self.input.chars().collect();
    let mut char_pos = self.input[..self.cursor].chars().count();

    // Skip any trailing whitespace at cursor
    while char_pos > 0 && chars[char_pos - 1].is_whitespace() {
      char_pos -= 1;
    }

    // Move backwards to find the start of the word
    while char_pos > 0 && !chars[char_pos - 1].is_whitespace() {
      char_pos -= 1;
    }

    // Convert character position back to byte position
    self.cursor = self.input.char_indices().nth(char_pos).map(|(pos, _)| pos).unwrap_or(0);
  }

  /// Move cursor by word boundaries forward
  fn move_cursor_word_right(&mut self) {
    if self.cursor >= self.input.len() {
      return;
    }

    let chars: Vec<char> = self.input.chars().collect();
    let mut char_pos = self.input[..self.cursor].chars().count();

    // Skip any leading whitespace at cursor
    while char_pos < chars.len() && chars[char_pos].is_whitespace() {
      char_pos += 1;
    }

    // Find the end of the word
    while char_pos < chars.len() && !chars[char_pos].is_whitespace() {
      char_pos += 1;
    }

    // Convert character position back to byte position
    self.cursor = self.input.char_indices().nth(char_pos).map(|(pos, _)| pos).unwrap_or(self.input.len());
  }

  /// Insert a character at the current cursor position
  fn insert_char(&mut self, chr: char) {
    if self.input.is_char_boundary(self.cursor) {
      self.input.insert(self.cursor, chr);
      self.cursor += chr.len_utf8();
    }
  }

  /// Delete the character before the cursor (backspace)
  fn delete_char_before_cursor(&mut self) {
    if self.cursor == 0 {
      return;
    }

    // Find the start of the character before cursor
    let mut char_start = self.cursor - 1;
    while char_start > 0 && !self.input.is_char_boundary(char_start) {
      char_start -= 1;
    }
    self.input.drain(char_start..self.cursor);
    self.cursor = char_start;
  }

  /// Delete the character at the cursor position (delete)
  fn delete_char_at_cursor(&mut self) {
    if self.cursor >= self.input.len() {
      return;
    }

    // Find the end of the character at cursor
    let mut char_end = self.cursor + 1;
    while char_end < self.input.len() && !self.input.is_char_boundary(char_end) {
      char_end += 1;
    }
    self.input.drain(self.cursor..char_end);
    // cursor stays at same position
  }

  /// Select all text (move cursor to end)
  fn select_all(&mut self) {
    self.cursor = self.input.len();
  }

  /// Clear all input text
  fn clear_input(&mut self) {
    self.input.clear();
    self.cursor = 0;
  }

  /// Get the current cursor position in character count (for display purposes)
  pub fn cursor_char_position(&self) -> usize {
    self.input[..self.cursor.min(self.input.len())].chars().count()
  }

  /// Ensure cursor is at a valid character boundary
  fn normalize_cursor(&mut self) {
    if self.cursor > self.input.len() {
      self.cursor = self.input.len();
    } else if !self.input.is_char_boundary(self.cursor) {
      // Find the next valid character boundary
      self.cursor = (self.cursor..=self.input.len())
        .find(|&pos| self.input.is_char_boundary(pos))
        .unwrap_or(self.input.len());
    }
  }

  /// Ensure we have a valid selection when items are available (async version)
  async fn ensure_selection(&mut self) {
    self.clamp_selection_async().await;
  }

  /// Ensure the selection is within bounds of the filtered list (async version)
  async fn clamp_selection_async(&mut self) {
    let filtered_items = self.read_items().await;
    if filtered_items.is_empty() {
      self.list_state.select(None);
    } else if let Some(selected) = self.list_state.selected() {
      if selected >= filtered_items.len() {
        self.list_state.select(Some(filtered_items.len() - 1));
      }
    } else {
      // No selection but items exist, select first one
      self.list_state.select(Some(0));
    }
  }

  async fn realpath(path: PathBuf) -> PathBuf {
    if !path.is_relative() {
      return path;
    }

    let Ok(canonicalized) = tokio::fs::canonicalize(&path).await else {
      return path;
    };
    canonicalized
      .components()
      .filter(|c| !matches!(c, std::path::Component::Prefix(_)))
      .collect()
  }

  fn realpath_blocking(path: PathBuf) -> PathBuf {
    if !path.is_relative() {
      return path;
    }

    let Ok(canonicalized) = path.canonicalize() else { return path };
    canonicalized
      .components()
      .filter(|c| !matches!(c, std::path::Component::Prefix(_)))
      .collect()
  }
}
