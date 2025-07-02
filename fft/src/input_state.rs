use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct InputState {
  cursor: usize,
  input: String,
}

impl Deref for InputState {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.input
  }
}

impl DerefMut for InputState {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.input
  }
}

impl InputState {
  pub fn new(input: impl Into<String>) -> Self {
    Self {
      cursor: 0,
      input: input.into(),
    }
  }
  pub fn set_cursor(&mut self, pos: usize) {
    self.cursor = pos;
  }

  pub fn cursor(&self) -> usize {
    self.cursor
  }
  pub fn push_str(&mut self, value: &str) {
    self.normalize_cursor();
    if self.input.is_char_boundary(self.cursor) {
      self.input.insert_str(self.cursor, value);
      self.cursor += value.len();

      // Normalize cursor after insertion
      self.normalize_cursor();
    }
  }

  pub fn left(&mut self) {
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

  pub fn normalize_cursor(&mut self) {
    if self.cursor > self.input.len() {
      self.cursor = self.input.len();
    } else if !self.input.is_char_boundary(self.cursor) {
      // Find the next valid character boundary
      self.cursor = (self.cursor..=self.input.len())
        .find(|&pos| self.input.is_char_boundary(pos))
        .unwrap_or(self.input.len());
    }
  }

  /// Move cursor to the next character boundary
  pub fn right(&mut self) {
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

  /// Insert a character at the current cursor position
  pub fn push(&mut self, chr: char) {
    if self.input.is_char_boundary(self.cursor) {
      self.input.insert(self.cursor, chr);
      self.cursor += chr.len_utf8();
    }
  }

  /// Delete the character before the cursor (backspace)
  pub fn backspace(&mut self) {
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
  pub fn delete(&mut self) {
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

  /// CTRL Backspace
  pub fn ctrl_backspace(&mut self) {
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

  /// CTRL Delete
  pub fn ctrl_delete(&mut self) {
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

  /// Move cursor by word boundaries
  pub fn move_left_word(&mut self) {
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

  pub fn move_right_word(&mut self) {
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
}

impl From<InputState> for String {
  fn from(value: InputState) -> Self {
    value.input
  }
}
