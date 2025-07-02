mod runner;
use runner::*;

mod explorer_content;
pub use explorer_content::*;

mod explorer;
pub use explorer::*;

mod explorer_state;
pub use explorer_state::*;

mod input_state;
pub use input_state::*;

/// Trait for extending `String` with additional functionalities.
/// This trait provides methods for manipulating strings in a way that is useful for text input handling.
/// All methods work with byte positions for proper UTF-8 handling.
pub trait StringExt {
  /// Insert a character at the given byte position.
  fn insert_char_at_byte(&mut self, byte_pos: usize, chr: char);
  /// Remove a character at the given byte position.
  fn remove_char_at_byte(&mut self, byte_pos: usize) -> Option<char>;
}

impl StringExt for String {
  fn insert_char_at_byte(&mut self, byte_pos: usize, chr: char) {
    if byte_pos <= self.len() && self.is_char_boundary(byte_pos) {
      self.insert(byte_pos, chr);
    }
  }

  fn remove_char_at_byte(&mut self, byte_pos: usize) -> Option<char> {
    if byte_pos >= self.len() || !self.is_char_boundary(byte_pos) {
      return None;
    }

    // Find the character at the given byte position
    let chr = self[byte_pos..].chars().next()?;
    let char_end = byte_pos + chr.len_utf8();

    // Ensure we don't go out of bounds
    if char_end <= self.len() {
      self.drain(byte_pos..char_end);
      Some(chr)
    } else {
      None
    }
  }
}

pub trait Fuzzier {
  /// Check if the string contains the pattern in a fuzzy manner.
  fn fuzzy_contains(&self, pattern: &str) -> bool
  where
    Self: AsRef<str>,
  {
    let str = self.as_ref();
    let mut pattern_chars = pattern.chars();
    let mut current_char = pattern_chars.next();

    for chr in str.chars() {
      if Some(chr) == current_char {
        current_char = pattern_chars.next();
      }
      if current_char.is_none() {
        return true;
      }
    }

    false
  }

  /// Get the fuzzy score of the string against a pattern.
  fn fuzzy_score(&self, pattern: &str) -> usize
  where
    Self: AsRef<str>,
  {
    let str = self.as_ref();
    let mut score = 0;
    let mut pattern_chars = pattern.chars();
    let mut current_char = pattern_chars.next();

    for chr in str.chars() {
      if Some(chr) == current_char {
        score += 1;
        current_char = pattern_chars.next();
      }
      if current_char.is_none() {
        break;
      }
    }
    score
  }
}
impl<T> Fuzzier for T where T: AsRef<str> {}
