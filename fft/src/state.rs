use super::*;
use crossterm::event::KeyEvent;
use helper::{RenderEvent, keys};
use ratatui::widgets::ListState;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Receiver;
use tokio::sync::{RwLock, watch};
use tokio::time::sleep;
use widget::ExplorerContent;

impl Default for ExplorerState {
  fn default() -> Self {
    let list: Arc<RwLock<Vec<ExplorerContent>>> = Default::default();
    let cwd = PathBuf::from("./");
    let (channels, new_channels) = Channels::new();

    // Send the initial directory to trigger scanning
    _ = channels.current_parrent_tx.send(Some(cwd.clone()));

    let runners = Runner::new(new_channels, list.clone());

    Self {
      runners,
      channels,
      cursor: 0,
      input: String::new(),
      cwd,
      list,
      list_state: ListState::default(),
    }
  }
}

pub struct ExplorerState {
  #[allow(dead_code)]
  pub(super) runners: Runner,
  pub(super) channels: Channels,

  pub(super) cursor: usize, // byte position in input string
  pub(super) input: String,

  pub(super) cwd: PathBuf,
  pub(super) list: Arc<RwLock<Vec<ExplorerContent>>>,

  pub(super) list_state: ListState,
}

impl ExplorerState {
  async fn update_watched_child(&mut self) {
    let selected_content = self.selected_content().await;
    self
      .channels
      .current_child_tx
      .send_modify(|current_child| *current_child = selected_content);
  }

  async fn update_watched_cwd(&mut self) {
    self.channels.current_child_tx.send_modify(|current_child| _ = current_child.take());

    // Notify the scanner of the new current working directory
    self
      .channels
      .current_parrent_tx
      .send_modify(|current_cwd| *current_cwd = Some(self.cwd.clone()));

    // Give the scanner a moment to start processing
    sleep(Duration::from_millis(16)).await;

    // Sync the new list contents and ensure selection
    self.ensure_selection().await;

    // Update the selected child if we have items
    let selected_child = if let Some(i) = self.list_state.selected() {
      self.read_items().await.get(i).cloned()
    } else {
      None
    };

    self
      .channels
      .current_child_tx
      .send_modify(|current_child| *current_child = selected_child);
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
      keys!(Up, NONE, Press) => {
        self.list_state.select_previous();
        self.update_watched_child().await;
      }
      keys!(Down, NONE, Press) => {
        self.list_state.select_next();
        self.update_watched_child().await;
      }
      keys!(Left, NONE, Press) => {
        self.move_cursor_left();
      }
      keys!(Right, NONE, Press) => {
        self.move_cursor_right();
      }

      keys!(Char(chr), NONE, Press) => {
        self.insert_char(chr);
        self.clamp_selection_async().await; // Use async version in key handler
      }
      keys!(Backspace, NONE, Press) => {
        self.delete_char_before_cursor();
        self.clamp_selection_async().await; // Use async version in key handler
      }
      keys!(Delete, NONE, Press) => {
        self.delete_char_at_cursor();
        self.clamp_selection_async().await; // Use async version in key handler
      }
      keys!(Backspace, CONTROL, Press) => {
        self.remove_word_backwards();
        self.clamp_selection_async().await; // Use async version in key handler
      }
      keys!(Delete, CONTROL, Press) => {
        self.remove_word_forwards();
        self.clamp_selection_async().await; // Use async version in key handler
      }
      keys!(Home, NONE, Press) => {
        self.move_cursor_home();
      }
      keys!(End, NONE, Press) => {
        self.move_cursor_end();
      }
      keys!(Left, CONTROL, Press) => {
        self.move_cursor_word_left();
      }
      keys!(Right, CONTROL, Press) => {
        self.move_cursor_word_right();
      }

      keys!(Esc, NONE, Press) => return Some(RenderEvent::canceled()),
      keys!(Enter, NONE, Press) => return Some(RenderEvent::handled()),
      keys!(Left, ALT, Press) => {
        if self.cwd.pop() {
          self.update_watched_cwd().await;
        }

        return None;
      }
      keys!(Right, ALT, Press) => {
        if let Some(content) = self.selected_content().await
          && !content.is_file()
        {
          self.cwd = content.as_path().to_path_buf();
          self.update_watched_cwd().await;
        } else {
          return None;
        }
      }

      keys!(Char('a'), CONTROL, Press) => {
        self.select_all();
      }
      keys!(Char('u'), CONTROL, Press) => {
        self.clear_input();
        self.clamp_selection_async().await; // Use async version in key handler
      }

      _ => {
        // No key matched, no need to render
        return None;
      }
    }

    // Always normalize cursor after text operations
    self.normalize_cursor();
    Some(RenderEvent::render())
  }

  pub(super) fn selected_content_blocking(&self) -> Option<ExplorerContent> {
    self.blocking_read_items().get(self.list_state.selected()?).cloned()
  }

  pub(super) fn blocking_read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.blocking_read();
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.as_cow().fuzzy_contains(self.input.as_str()))
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

  pub(super) async fn read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.read().await;
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.as_cow().fuzzy_contains(self.input.as_str()))
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

  pub async fn get(self) -> Option<PathBuf> {
    self
      .read_items()
      .await
      .get(self.list_state.selected()?)
      .map(|content| content.as_path().to_path_buf())
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

  /// Ensure the selection is within bounds of the filtered list (blocking version for widget rendering)
  /// This should only be called from widget render contexts where blocking is safe
  #[allow(dead_code)]
  fn clamp_selection(&mut self) {
    let filtered_items = self.blocking_read_items();
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

  /// Debug method to check the current state of the list
  #[allow(dead_code)]
  pub fn debug_list_state(&self) -> (usize, usize, bool) {
    let list = self.list.blocking_read();
    let filtered_count = self.blocking_read_items().len();
    let has_selection = self.list_state.selected().is_some();
    (list.len(), filtered_count, has_selection)
  }
}

pub struct Channels {
  pub(super) current_parrent_tx: watch::Sender<Option<PathBuf>>,
  pub(super) current_child_tx: watch::Sender<Option<ExplorerContent>>,
}
pub struct NewChannels {
  pub parrent_watch: Receiver<Option<PathBuf>>,
  pub child_watch: Receiver<Option<ExplorerContent>>,
}

impl Channels {
  pub fn new() -> (Channels, NewChannels) {
    let (current_parrent_tx, current_parrent_rx) = watch::channel(Option::<PathBuf>::None);
    let (current_child_tx, current_child_rx) = watch::channel(Option::<ExplorerContent>::None);
    (
      Self {
        current_parrent_tx,
        current_child_tx,
      },
      NewChannels {
        parrent_watch: current_parrent_rx,
        child_watch: current_child_rx,
      },
    )
  }
}
