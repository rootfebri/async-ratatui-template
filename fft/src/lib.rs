mod runner;
use runner::*;

mod char_stream;
pub mod state;
pub mod widget;
pub use char_stream::*;

/// Trait for extending `String` with additional functionalities.
/// This trait provides methods for manipulating strings in a way that is useful for text input handling,
/// such as pushing and removing characters, words, and fuzzy matching.
pub trait StringExt {
  /// Push a character into the string at the current position.
  fn push_char(&mut self, chr: char, char_pos: &mut usize);
  /// Remove a character at the current position.
  fn remove_char(&mut self, char_pos: usize) -> Option<char>;
  /// Remove a word backwards from the current position.
  /// Returns the number of characters removed and a SeekFrom indicating the new cursor position.
  fn remove_word_backwards(&mut self, char_pos: &mut usize);
  /// Remove a word forwards from the current position.
  /// Returns the number of characters removed and a SeekFrom indicating cursor movement.
  fn remove_word_forwards(&mut self, char_pos: usize);
}

impl StringExt for String {
  fn push_char(&mut self, chr: char, char_pos: &mut usize) {
    // Convert character position to byte position for insertion
    let byte_pos = self.char_indices().nth(*char_pos).map(|(pos, _)| pos).unwrap_or(self.len());

    self.insert(byte_pos, chr);
    *char_pos += 1;
  }

  fn remove_char(&mut self, char_pos: usize) -> Option<char> {
    // Find the character at the given position
    if let Some((byte_pos, chr)) = self.char_indices().nth(char_pos) {
      self.remove(byte_pos);
      Some(chr)
    } else {
      None
    }
  }

  fn remove_word_backwards(&mut self, char_pos: &mut usize) {
    if self.is_empty() || *char_pos == 0 {
      return;
    }

    let chars: Vec<char> = self.chars().collect();
    let mut end_pos = (*char_pos).min(chars.len());
    let start_pos = end_pos;

    // Move backwards to skip any trailing whitespace at cursor
    while end_pos > 0 && chars[end_pos - 1].is_whitespace() {
      end_pos -= 1;
    }

    // Move backwards to find the start of the word
    while end_pos > 0 && !chars[end_pos - 1].is_whitespace() {
      end_pos -= 1;
    }

    let removed_count = start_pos - end_pos;

    if removed_count == 0 {
      return;
    }
    // Convert character positions to byte positions
    let start_byte = self.char_indices().nth(end_pos).map(|(pos, _)| pos).unwrap_or(0);
    let end_byte = self.char_indices().nth(start_pos).map(|(pos, _)| pos).unwrap_or(self.len());

    self.drain(start_byte..end_byte);

    // Return absolute position to the start of removed text
    *char_pos = end_pos;
  }

  fn remove_word_forwards(&mut self, char_pos: usize) {
    if self.is_empty() {
      return;
    }

    let chars: Vec<char> = self.chars().collect();
    let mut start_pos = char_pos.min(chars.len());
    let original_start = start_pos;

    // Skip any leading whitespace at cursor
    while start_pos < chars.len() && chars[start_pos].is_whitespace() {
      start_pos += 1;
    }

    // Find the end of the word
    let mut end_pos = start_pos;
    while end_pos < chars.len() && !chars[end_pos].is_whitespace() {
      end_pos += 1;
    }

    let removed_count = end_pos - original_start;

    if removed_count > 0 {
      // Convert character positions to byte positions
      let start_byte = self.char_indices().nth(original_start).map(|(pos, _)| pos).unwrap_or(self.len());
      let end_byte = self.char_indices().nth(end_pos).map(|(pos, _)| pos).unwrap_or(self.len());

      if start_byte < self.len() {
        self.drain(start_byte..end_byte);
      }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_push_char() {
    let mut s = String::from("hello");
    let mut pos = 2;
    s.push_char('X', &mut pos);
    assert_eq!(s, "heXllo");
    assert_eq!(pos, 3);

    let mut s = String::new();
    let mut pos = 0;
    s.push_char('A', &mut pos);
    assert_eq!(s, "A");
    assert_eq!(pos, 1);
  }

  #[test]
  fn test_remove_char() {
    let mut s = String::from("hello");
    assert_eq!(s.remove_char(1), Some('e'));
    assert_eq!(s, "hllo");
    assert_eq!(s.remove_char(10), None);
  }

  #[test]
  fn test_remove_word_backwards() {
    let mut pos = 11;
    let mut s = String::from("hello world test");
    s.remove_word_backwards(&mut pos); // Position after "world"
    assert_eq!(s, "hello  test");
    assert_eq!(pos, 6);
  }

  #[test]
  fn test_remove_word_forwards() {
    let mut s = String::from("hello world test");
    s.remove_word_forwards(6); // Position at start of "world"
    assert_eq!(s, "hello test");
  }

  #[test]
  fn test_fuzzy_contains() {
    let s = String::from("hello world");
    assert!(s.fuzzy_contains("hlo"));
    assert!(s.fuzzy_contains("helloworld"));
    assert!(!s.fuzzy_contains("xyz"));
  }

  #[test]
  fn test_fuzzy_score() {
    let s = String::from("hello world");
    assert_eq!(s.fuzzy_score("hlo"), 3);
    assert_eq!(s.fuzzy_score("hello"), 5);
    assert_eq!(s.fuzzy_score("xyz"), 0);
  }
}
