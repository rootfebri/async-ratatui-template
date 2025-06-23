use crossterm::event::Event;
use helper::UnhandledEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::Paragraph;

use crate::ui::{clear, fix_center};
use crate::widgets::{Alert, Confirmation, Input};

pub enum Popup {
  Input(Input),
  Confirmation(Confirmation),
  Warning(Paragraph<'static>),
  Alert(Alert),
}

impl Popup {
  pub fn handle_event(&mut self, event: &Event) -> Option<UnhandledEvent> {
    match *self {
      Popup::Input(ref mut input) => input.handle_event(event),
      Popup::Confirmation(_) => todo!(),
      Popup::Warning(_) => todo!(),
      Popup::Alert(ref mut alert) => alert.handle_event(event),
    }
  }

  pub fn area(&self, area: Rect) -> Rect {
    match *self {
      Popup::Input(_) => fix_center(area, 65, 25),
      Popup::Confirmation(_) => fix_center(area, 45, 25),
      Popup::Warning(_) => fix_center(area, 15, 15),
      Popup::Alert(_) => fix_center(area, 25, 25),
    }
  }
}

impl Widget for &Popup {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let area = self.area(area);
    clear(area, buf);

    match self {
      Popup::Input(widget) => widget.render(area, buf),
      Popup::Confirmation(widget) => widget.render(area, buf),
      Popup::Warning(widget) => widget.render(area, buf),
      Popup::Alert(widget) => widget.render(area, buf),
    }
  }
}
