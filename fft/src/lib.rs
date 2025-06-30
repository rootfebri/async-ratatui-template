mod runner;
use runner::*;

pub mod state;
pub mod widget;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_insert_char_at_byte() {
    let mut s = String::from("hello");
    s.insert_char_at_byte(2, 'X');
    assert_eq!(s, "heXllo");

    let mut s = String::new();
    s.insert_char_at_byte(0, 'A');
    assert_eq!(s, "A");
  }

  #[test]
  fn test_remove_char_at_byte() {
    let mut s = String::from("hello");
    assert_eq!(s.remove_char_at_byte(1), Some('e'));
    assert_eq!(s, "hllo");
    assert_eq!(s.remove_char_at_byte(10), None);
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

  #[test]
  fn test_insert_char_at_byte_utf8() {
    let mut s = String::from("héllo");
    // In "héllo", 'h' is at 0, 'é' is at 1-2, 'l' is at 3
    s.insert_char_at_byte(3, 'X'); // Insert at byte position 3 (before first 'l')
    assert_eq!(s, "héXllo");

    let mut s = String::from("🚀test");
    // In "🚀test", emoji is 4 bytes (0-3), 't' starts at 4
    s.insert_char_at_byte(4, 'X'); // Insert after emoji (4 bytes)
    assert_eq!(s, "🚀Xtest");
  }

  #[test]
  fn test_remove_char_at_byte_utf8() {
    let mut s = String::from("héllo");
    // In "héllo", 'é' starts at byte 1
    assert_eq!(s.remove_char_at_byte(1), Some('é')); // Remove é (2 bytes)
    assert_eq!(s, "hllo");

    let mut s = String::from("🚀test");
    assert_eq!(s.remove_char_at_byte(0), Some('🚀')); // Remove emoji (4 bytes)
    assert_eq!(s, "test");
  }

  #[test]
  fn test_remove_char_at_byte_bounds() {
    let mut s = String::from("test");
    assert_eq!(s.remove_char_at_byte(4), None); // Out of bounds
    assert_eq!(s.remove_char_at_byte(10), None); // Way out of bounds

    // Test removing from invalid byte position within UTF-8 character
    let mut s = String::from("héllo");
    // Byte 2 is in the middle of 'é' character, so it should return None
    assert_eq!(s.remove_char_at_byte(2), None);
  }
}
