#[macro_export]
macro_rules! __keys_pattern_match {
  ($key1:ident($val1:tt) | $key2:ident($val2:tt)) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key1($val1) | ::crossterm::event::KeyCode::$key2($val2),
      ..
    }
  };
}

#[macro_export]
macro_rules! __keys_single {
  ($key:ident($val:tt)) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key($val),
      ..
    }
  };
  ($key:ident) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key,
      ..
    }
  };
}

#[macro_export]
macro_rules! __keys_with_mods {
  ($key:ident($val:tt), $($mods:ident)|+) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key($val),
      modifiers: $(::crossterm::event::KeyModifiers::$mods)|+,
      ..
    }
  };
  ($key:ident, $($mods:ident)|+) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key,
      modifiers: $(::crossterm::event::KeyModifiers::$mods)|+,
      ..
    }
  };
}

#[macro_export]
macro_rules! __keys_with_mods_kind {
  ($key:ident($val:tt), $($mods:ident)|+, $kind:ident) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key($val),
      modifiers: $(::crossterm::event::KeyModifiers::$mods)|+,
      kind: ::crossterm::event::KeyEventKind::$kind,
      ..
    }
  };
  ($key:ident, $($mods:ident)|+, $kind:ident) => {
    ::crossterm::event::KeyEvent {
      code: ::crossterm::event::KeyCode::$key,
      modifiers: $(::crossterm::event::KeyModifiers::$mods)|+,
      kind: ::crossterm::event::KeyEventKind::$kind,
      ..
    }
  };
}

/// Token Tree Implementation for Random Position Key Patterns
///
/// This module provides a flexible macro system that allows key patterns
/// to be specified in different orders, supporting:
///
/// 1. Traditional patterns: keys!(Char('a'), CONTROL | META)
/// 2. Random position patterns: keys!(CONTROL | META, Char('a'))
/// 3. Modifier-only patterns: keys!(CONTROL), keys!(ALT)
/// 4. KeyCode alternatives: keys!(Char('s') | Char('g'))
///
/// # Examples:
/// ```no_run
/// xterm_defined::keys!(Enter);                        // Just keycode
/// xterm_defined::keys!(Char('a'));                    // Keycode with value
/// xterm_defined::keys!(F(1));                         // Function key
/// xterm_defined::keys!(Char('c'), CONTROL);           // Traditional order
/// xterm_defined::keys!(CONTROL, Char('c'));           // Random position (modifier first)
/// xterm_defined::keys!(ALT | SHIFT, Enter);           // Multiple modifiers first
/// xterm_defined::keys!(CONTROL);                      // Modifier only (matches any keycode with CONTROL)
/// xterm_defined::keys!(Char('a'), CONTROL, Press);    // With event kind
///```
///
/// # Note
///
/// The macro creates identical KeyEvent patterns regardless of argument order.
/// Compiler warnings about unreachable patterns are expected when using both
/// traditional and random position patterns in the same match statement.
#[macro_export]
macro_rules! keys {
  // Priority order: Most specific patterns first to avoid ambiguity

  // Multiple KeyCodes with pipe separator (e.g., Char('s') | Char('g'))
  ($key1:ident($val1:tt) | $key2:ident($val2:tt)) => {
    $crate::__keys_pattern_match! { $key1($val1) | $key2($val2) }
  };

  // KeyCode with modifiers and kind (most specific - 3 parts)
  ($key:ident($val:tt), $($mods:ident)|+, $kind:ident) => {
    $crate::__keys_with_mods_kind! { $key($val), $($mods)|+, $kind }
  };
  ($key:ident, $($mods:ident)|+, $kind:ident) => {
    $crate::__keys_with_mods_kind! { $key, $($mods)|+, $kind }
  };

  // Random position: modifiers first, then keycode (2 parts)
  ($($mods:ident)|+, $key:ident($val:tt)) => {
    $crate::__keys_with_mods! { $key($val), $($mods)|+ }
  };
  ($($mods:ident)|+, $key:ident) => {
    $crate::__keys_with_mods! { $key, $($mods)|+ }
  };

  // Traditional: KeyCode with modifiers (2 parts)
  ($key:ident($val:tt), $($mods:ident)|+) => {
    $crate::__keys_with_mods! { $key($val), $($mods)|+ }
  };
  ($key:ident, $($mods:ident)|+) => {
    $crate::__keys_with_mods! { $key, $($mods)|+ }
  };

  // Single modifiers only (1 part)
  (CONTROL) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::CONTROL,
      ..
    }
  };
  (ALT) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::ALT,
      ..
    }
  };
  (META) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::META,
      ..
    }
  };
  (SHIFT) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::SHIFT,
      ..
    }
  };
  (SUPER) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::SUPER,
      ..
    }
  };
  (HYPER) => {
    ::crossterm::event::KeyEvent {
      modifiers: ::crossterm::event::KeyModifiers::HYPER,
      ..
    }
  };

  // Single KeyCode patterns (1 part) - must come last
  ($key:ident($val:tt)) => {
    $crate::__keys_single! { $key($val) }
  };
  ($key:ident) => {
    $crate::__keys_single! { $key }
  };
}
