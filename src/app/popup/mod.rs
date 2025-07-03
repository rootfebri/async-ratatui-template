use std::cell::RefCell;
use std::ops::DerefMut;

use crossterm::event::Event;
use fft::{Explorer, ExplorerState};
use helper::RenderEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Constraint, Widget};

use crate::ui::center_constraints;
use crate::widgets::{Confirmation, Input};

pub enum Popup {
  Input(Input),
  Confirmation(Confirmation),
  FileExplorer(RefCell<ExplorerState>),
}

impl Popup {
  pub async fn handle_event(&mut self, event: &Event) -> Option<RenderEvent> {
    match *self {
      Popup::Input(ref mut input) => input.handle_event(event),
      Popup::Confirmation(ref mut modal) => modal.handle_key(event.as_key_event()?),
      Popup::FileExplorer(ref mut explore_state) => match event {
        Event::Key(key) => explore_state.get_mut().handle_key(*key).await,
        Event::Paste(content) => Some(explore_state.get_mut().handle_paste(content)),
        _ => None,
      },
    }
  }

  pub fn area(&self, area: Rect) -> Rect {
    match *self {
      Popup::Input(_) => center_constraints(area, Constraint::Min(55), Constraint::Length(15)),
      Popup::Confirmation(..) => center_constraints(area, Constraint::Length(55), Constraint::Length(7)),
      Popup::FileExplorer(_) => center_constraints(area, Constraint::Percentage(80), Constraint::Percentage(90)),
    }
  }
}

impl Widget for &Popup {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let area = self.area(area);

    match self {
      Popup::Input(widget) => widget.render(area, buf),
      Popup::Confirmation(modal) => modal.render(area, buf),
      Popup::FileExplorer(state) => Explorer::new(state.borrow_mut().deref_mut()).render(area, buf),
    }
  }
}
