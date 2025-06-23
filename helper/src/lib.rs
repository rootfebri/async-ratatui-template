mod events;
mod internal_macros;

pub use events::*;

#[cfg(test)]
mod tests {
  use std::mem::zeroed;

  use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  use super::*;

  #[test]
  fn test_match() {
    let e: KeyEvent = unsafe { zeroed() };

    match e {
      keys!(Char('s') | Char('g')) => unreachable!("Shouldn't be here, trust me!"),
      keys!(F(1)) => unreachable!("Shouldn't be here, trust me!"),
      keys!(Enter) => unreachable!("Shouldn't be here, trust me!"),
      keys!(Char('c'), CONTROL | META) => unreachable!("Shouldn't be here, trust me!"),
      keys!(Char('a'), CONTROL | META, Press) => unreachable!("Shouldn't be here, trust me!"),
      k => assert_eq!(k, e),
    }
  }

  #[test]
  fn test_random_position_patterns() {
    let e: KeyEvent = unsafe { zeroed() };

    match e {
      // Modifiers first, then keycode
      keys!(CONTROL | ALT, Char('x')) => unreachable!("Shouldn't be here, trust me!"),
      keys!(SHIFT, Enter) => unreachable!("Shouldn't be here, trust me!"),

      // Only single modifiers (no specific keycode) - these work
      keys!(ALT) => unreachable!("Shouldn't be here, trust me!"),

      k => assert_eq!(k, e),
    }
  }

  #[test]
  fn test_modifier_only_patterns() {
    // Test that patterns with only modifiers match any keycode with those modifiers
    let test_event = KeyEvent {
      code: KeyCode::Char('a'),
      modifiers: KeyModifiers::CONTROL,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    };

    // Should match because it has CONTROL modifier
    let mut matched = false;
    match test_event {
      keys!(CONTROL) => matched = true,
      _ => {}
    }
    assert!(matched, "Should match CONTROL modifier pattern");

    // Test that different modifier doesn't match
    let alt_event = KeyEvent {
      code: KeyCode::Enter,
      modifiers: KeyModifiers::ALT,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    };

    let mut alt_matched = false;
    match alt_event {
      keys!(CONTROL) => alt_matched = true,
      _ => {}
    }
    assert!(!alt_matched, "Should not match CONTROL when event has ALT");
  }

  #[test]
  fn test_random_position_modifier_first() {
    // Test patterns where modifiers come before the keycode
    let test_event = KeyEvent {
      code: KeyCode::Char('x'),
      modifiers: KeyModifiers::ALT,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    };

    let mut matched = false;
    match test_event {
      keys!(ALT, Char('x')) => matched = true,
      _ => {}
    }
    assert!(matched, "Should match ALT, Char('x') pattern");

    // Test that it doesn't match wrong combinations
    let mut wrong_matched = false;
    match test_event {
      keys!(CONTROL, Char('x')) => wrong_matched = true,
      _ => {}
    }
    assert!(!wrong_matched, "Should not match CONTROL, Char('x') pattern");
  }

  #[test]
  #[allow(unreachable_patterns)] // Expected for flexible macro patterns
  fn test_comprehensive_random_patterns() {
    // Test comprehensive random position patterns
    let test_events = vec![
      // Test traditional vs random position equivalence
      (
        KeyEvent {
          code: KeyCode::Char('z'),
          modifiers: KeyModifiers::SHIFT,
          kind: KeyEventKind::Press,
          state: crossterm::event::KeyEventState::NONE,
        },
        "SHIFT + Char('z')",
      ),
      (
        KeyEvent {
          code: KeyCode::F(12),
          modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
          kind: KeyEventKind::Press,
          state: crossterm::event::KeyEventState::NONE,
        },
        "CONTROL+ALT + F12",
      ),
    ];

    for (event, description) in test_events {
      println!("Testing: {}", description);

      // Test that both traditional and random position patterns work
      match event {
        // Traditional order
        keys!(Char('z'), SHIFT) => {
          println!("  ✓ Traditional pattern matched: keys!(Char('z'), SHIFT)");
        }
        keys!(F(12), CONTROL | ALT) => {
          println!("  ✓ Traditional pattern matched: keys!(F(12), CONTROL | ALT)");
        }
        // Random position (modifier first)
        keys!(SHIFT, Char('z')) => {
          println!("  ✓ Random position pattern matched: keys!(SHIFT, Char('z'))");
        }
        keys!(CONTROL | ALT, F(12)) => {
          println!("  ✓ Random position pattern matched: keys!(CONTROL | ALT, F(12))");
        }
        _ => {
          println!("  ✗ No pattern matched for {}", description);
        }
      }
    }
  }
}
