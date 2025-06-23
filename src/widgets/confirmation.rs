#![allow(dead_code)]

use std::cell::RefCell;

use crossterm::event::Event;
use helper::keys;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::prelude::Stylize;
use ratatui::style::Color;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::ui::blk;

#[derive(Eq, PartialEq)]
pub struct Confirmation {
  prompt: String,
  value: bool,
  last_render: Option<RefCell<[Rect; 3]>>,
}

impl Confirmation {
  pub fn new(prompt: String, value: bool) -> Self {
    Self {
      prompt,
      last_render: None,
      value,
    }
  }

  pub fn handle_event(&mut self, event: Event) -> Option<Event> {
    match event {
      Event::Key(keys!(Char('Y'), SHIFT, Press)) => (),
      Event::Mouse(_mouse) => (),
      _ => (),
    }

    Some(event)
  }

  pub fn area(&self, area: Rect) -> [Rect; 3] {
    let [input, footer] = Layout::vertical([Constraint::Max(4), Constraint::Length(3)]).areas(area);
    let footer = Layout::horizontal([Constraint::Fill(1); 2])
      .spacing(Spacing::Space(4))
      .horizontal_margin(4)
      .split(footer);

    let areas = [input, footer[0], footer[1]];

    if let Some(ref last_render) = self.last_render {
      last_render.replace(areas);
    }

    areas
  }

  fn confirm_block(&self) -> Block {
    let spans = vec![Span::raw(" "), Span::raw(self.prompt.as_str()), Span::raw(" ")];
    let line = Line::from(spans).centered();
    blk().title_top(line).fg(Color::Rgb(255, 123, 0))
  }

  fn draw_button<'a>(&'a self, text: impl Into<Text<'a>>, bg: Color, area: Rect, buf: &mut Buffer) {
    let block = blk();
    let text = text.into().centered().fg(Color::White).bg(bg);
    Paragraph::new(text).block(block).bg(Color::Yellow).render(area, buf)
  }
}

impl Widget for &Confirmation {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let [input, _confirm, _cancel] = self.area(area);
    Paragraph::new(self.prompt.as_str()).block(self.confirm_block()).render(input, buf);
  }
}
