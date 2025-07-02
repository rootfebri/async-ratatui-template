use ratatui::prelude::*;

mod confirmation;
pub use confirmation::*;

mod input;
pub use input::*;

mod statistic;
pub use statistic::*;

mod logs;
pub use logs::*;

mod alert;
pub use alert::*;

mod timestamp;
pub use timestamp::*;

mod log;
pub use log::*;

pub mod pulse;

pub fn str_with_hotkey(value: &str) -> Option<(&str, &str)> {
  if value.is_empty() {
    return None;
  }

  let initial_len = value.chars().next().unwrap().len_utf8();
  let initial = &value[..initial_len];
  let rest = &value[initial_len..];

  Some((initial, rest))
}
pub fn line_with_hotkey<C: Into<Option<Color>>>(value: &str, spaced: bool, hkc: C, line_color: C) -> Line {
  let hkc = hkc.into().unwrap_or(Color::White);
  let line_color = line_color.into().unwrap_or(Color::White);

  if value.is_empty() {
    Line::raw(value)
  } else {
    let first_char_len = value.chars().next().unwrap().len_utf8();
    let initial = &value[..first_char_len];
    let rest = &value[first_char_len..];

    let spaced_opening = if spaced { Span::styled(" [", hkc) } else { Span::styled("[", hkc) };
    let spaced_closing = if spaced { Span::styled(" ", hkc) } else { Span::raw("") };

    Line::from_iter([
      spaced_opening,
      Span::styled(initial, hkc).italic(),
      Span::styled("]", hkc),
      Span::styled(rest, line_color),
      spaced_closing,
    ])
  }
}
