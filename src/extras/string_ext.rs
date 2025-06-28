pub enum FuzzyRespect {
  CaseSensitive,
  CaseInsensitive,
  RespectCaseWhen,
}

/// Trait for extending `String` with additional functionalities.
/// This trait provides methods for manipulating strings in a way that is useful for text input handling, such as pushing and removing characters, words, and fuzzy matching.
pub trait StringExt {
  /// Push a character into the string at the current position.
  fn push_char(&mut self, chr: char, char_pos: &mut usize);

  /// Remove a character after the current position.
  fn remove_char(&mut self, char_pos: usize) -> Option<char>;

  /// Remove a character before the current position.
  fn remove_word_backwards(&mut self, char_pos: usize) -> usize;

  /// Remove a word after the current position.
  fn remove_word_afterwards(&mut self, char_pos: usize) -> usize;

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

impl StringExt for String {
  fn push_char(&mut self, chr: char, char_pos: &mut usize) {
    // Since we get char position as a mutable reference, we can use it to insert the character at the correct position.

    // If the position is 0, we simply prepend the character.
    if *char_pos == 0 || self.is_empty() {
      self.push(chr);
      *char_pos += 1;
      return;
    }

    let mut chars = self.chars().collect::<Vec<_>>();
    chars.insert(*char_pos - 1, chr);
    *self = chars.into_iter().collect();
    *char_pos += 1;
  }

  fn remove_char(&mut self, char_pos: usize) -> Option<char> {
    if self.is_empty() {
      return None; // No character to remove after the current position
    }

    let mut removed_char = None;
    *self = self
      .chars()
      .enumerate()
      .filter_map(|(i, chr)| {
        if char_pos == i {
          removed_char = Some(chr);
          None
        } else {
          Some(chr)
        }
      })
      .collect();

    removed_char
  }

  fn remove_word_backwards(&mut self, char_pos: usize) -> usize {
    if self.is_empty() {
      return 0;
    }

    let mut removed_chars = vec![];
    let mut indexes = vec![];
    let mut chars = self.chars().collect::<Vec<_>>();

    while let Some((index, chr)) = chars.iter().enumerate().nth(char_pos) {
      indexes.push(index);

      if chr.is_ascii_whitespace() {
        break;
      }
    }

    indexes.sort();
    for index in indexes.into_iter().rev() {
      removed_chars.push(chars.remove(index));
    }

    *self = chars.into_iter().collect();

    removed_chars.reverse();
    removed_chars.len()
  }

  fn remove_word_afterwards(&mut self, char_pos: usize) -> usize {
    if self.is_empty() {
      return 0;
    }

    let mut chars = self.chars().collect::<Vec<_>>();
    if chars.len() <= char_pos {
      return 0;
    }

    let mut removed = String::with_capacity(char_pos);
    let mut collected = false;
    *self = chars
      .into_iter()
      .enumerate()
      .filter_map(|(index, chr)| {
        if index <= char_pos || collected {
          return Some(chr);
        }

        if chr == ' ' {
          collected = true;
        }

        removed.push(chr);
        None
      })
      .collect();

    removed.len()
  }
}
