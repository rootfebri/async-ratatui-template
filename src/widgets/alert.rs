use std::ops::Deref;
use std::rc::Rc;

use crossterm::event::Event;
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

use crate::areas::KnownArea;
use crate::ui::{blk, clear};

pub struct Alert {
  title: Rc<str>,
  content: Vec<Rc<str>>,
  known_area: KnownArea,
}

impl Alert {
  pub fn new(title: impl Into<Rc<str>>, content: impl Iterator<Item = impl Into<Rc<str>>>) -> Self {
    Self {
      title: title.into(),
      content: Vec::from_iter(content.map(Into::into)),
      known_area: Default::default(),
    }
  }

  pub fn add_line(mut self, line: impl Into<Rc<str>>) -> Self {
    self.content.push(line.into());
    self
  }

  pub fn handle_event(&mut self, event: &Event) -> Option<RenderEvent> {
    match event {
      Event::Key(keys!(Char('y'), NONE, Press) | keys!(Enter, NONE, Press)) => Some(RenderEvent::handled()),
      _ => None,
    }
  }

  fn as_lines(&self) -> impl Iterator<Item = Line> {
    self
      .content
      .iter()
      .map(Deref::deref)
      .map(str::as_bytes)
      .map(String::from_utf8_lossy)
      .map(Line::from)
  }

  fn line_controls(&self) -> Line {
    let spans = vec![Span::raw("[Y/ENTER]").blue(), Span::raw(" "), Span::raw("OK")];
    Line::from(spans).centered()
  }
}

impl Widget for &Alert {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    clear(area, buf);
    self.known_area.replace(area);

    let block = blk()
      .title_top(String::from_utf8_lossy(self.title.as_bytes()))
      .title_bottom(self.line_controls())
      .fg(Color::Rgb(255, 123, 0));

    Paragraph::new(Text::from_iter(self.as_lines()))
      .block(block)
      .wrap(Wrap { trim: false })
      .render(area, buf);
  }
}
