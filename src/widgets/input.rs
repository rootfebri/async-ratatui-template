use crossterm::event::Event;
use helper::{UnhandledEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Widget};
use ratatui::style::Color;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::Area;
use crate::areas::KnownArea;
use crate::ui::blk;

#[derive(Debug)]
pub struct Input {
  cursor: usize,
  value: String,
  label: String,
  placeholder: String,
  known_area: KnownArea,
}

impl Input {
  pub fn new(label: impl Into<Option<String>>, placeholder: impl Into<Option<String>>) -> Self {
    Self {
      cursor: 0,
      value: String::new(),
      label: label.into().unwrap_or(String::from("Input")).to_string(),
      placeholder: placeholder.into().unwrap_or(String::from("Start typing..")),
      known_area: Default::default(),
    }
  }

  pub fn handle_event(&mut self, event: &Event) -> Option<UnhandledEvent> {
    match event {
      Event::Key(keys!(Backspace, NONE, Press)) => {
        if self.value.is_empty() || self.cursor == 0 && self.value.chars().nth(self.cursor).is_none() {
          return Some(UnhandledEvent::no_ops());
        }

        self.cursor -= 1;
        self.value = self
          .value
          .chars()
          .enumerate()
          .filter(|&(i, _)| i != self.cursor)
          .map(|(_, chr)| chr)
          .collect();

        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Delete, NONE, Press)) => {
        if self.value.is_empty() || self.cursor >= self.value.len() {
          return Some(UnhandledEvent::no_ops());
        }

        self.value = self
          .value
          .chars()
          .enumerate()
          .filter(|&(i, _)| i != self.cursor)
          .map(|(_, chr)| chr)
          .collect();

        if self.cursor > self.value.len() {
          self.cursor = self.value.len();
        }

        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Enter, NONE, Press)) => Some(UnhandledEvent::handled()),
      Event::Key(keys!(Esc, NONE, Press)) => Some(UnhandledEvent::canceled()),
      Event::Key(keys!(Left, NONE, Press)) => {
        self.cursor -= 1;
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Right, NONE, Press)) => {
        self.cursor += 1;
        Some(UnhandledEvent::render())
      }
      Event::Key(keys!(Char(chr), NONE, Press)) => {
        self.value.push(*chr);
        self.cursor += 1;

        Some(UnhandledEvent::render())
      }
      Event::Paste(content) => {
        self.paste(content);
        Some(UnhandledEvent::render())
      }
      _ => None,
    }
  }

  fn paste(&mut self, content: impl AsRef<str>) {
    for (i, chr) in content.as_ref().chars().enumerate() {
      self.value.insert(self.cursor + i, chr)
    }
  }

  pub fn value(self) -> String {
    self.value
  }

  fn draw_label(&self) -> Line {
    Line::raw(&self.label).left_aligned().fg(Color::LightGreen)
  }
  fn draw_block(&self) -> Block {
    use ratatui::symbols::block::ONE_EIGHTH as I;

    let submit: Line = [
      Span::raw("[ENTER]").light_green(),
      Span::raw("Submit"),
      Span::raw(" "),
      Span::raw(I),
      Span::raw(" "),
      Span::raw("[ESC]").light_red(),
      Span::raw("Cancel"),
    ]
    .into_iter()
    .collect();

    blk().title_top(self.draw_label()).title_bottom(submit.centered())
  }

  pub fn horizontal_scroll(&self, width: usize) -> u16 {
    if self.cursor > width { self.cursor as u16 } else { 0 }
  }

  #[inline(always)]
  fn cursor(&self) -> Span {
    Span::raw(ratatui::symbols::line::VERTICAL).fg(Color::LightYellow)
  }
  fn draw_input(&self, area: Area, buf: &mut Buffer) {
    let block = self.draw_block();

    let spans: Vec<Span> = if self.value.is_empty() {
      vec![Span::from(self.placeholder.as_str()).fg(Color::DarkGray), self.cursor()]
    } else if self.cursor >= self.value.len() {
      vec![Span::from(self.value.as_str()), self.cursor()]
    } else {
      let left = &self.value[..=self.cursor];
      let right = &self.value[self.cursor..];
      vec![Span::from(left), self.cursor(), Span::from(right)]
    };

    Paragraph::new(Text::from_iter(spans))
      .block(block)
      .wrap(Wrap { trim: true })
      .render(area, buf);
  }
}

impl Widget for &Input {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    self.known_area.replace(area);
    self.draw_input(area, buf)
  }
}
