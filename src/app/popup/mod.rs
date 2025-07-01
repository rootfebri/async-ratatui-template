use crate::ui::{center_constraints, fix_center};
use crate::widgets::{Alert, Input};
use crossterm::event::Event;
use fft::Explorer;
use fft::ExplorerState;
use helper::RenderEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Constraint, Widget};
use ratatui::widgets::Paragraph;
use std::cell::RefCell;
use std::ops::DerefMut;

pub enum Popup {
  Input(Input),
  Confirmation(Paragraph<'static>),
  Warning(Paragraph<'static>),
  Alert(Alert),
  FileExplorer(RefCell<ExplorerState>),
}

impl Popup {
  pub async fn handle_event(&mut self, event: &Event) -> Option<RenderEvent> {
    match *self {
      Popup::Input(ref mut input) => input.handle_event(event),
      Popup::Confirmation(_) => todo!(),
      Popup::Warning(_) => todo!(),
      Popup::Alert(ref mut alert) => alert.handle_event(event),
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
      Popup::Confirmation(_) => fix_center(area, 45, 25),
      Popup::Warning(_) => fix_center(area, 15, 15),
      Popup::Alert(_) => fix_center(area, 25, 25),
      Popup::FileExplorer(_) => {
        area
        /*center_constraints(area, Constraint::Percentage(85), Constraint::Length(85))*/
      }
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
      Popup::Confirmation(widget) => widget.render(area, buf),
      Popup::Warning(widget) => widget.render(area, buf),
      Popup::Alert(widget) => widget.render(area, buf),
      Popup::FileExplorer(state) => Explorer::new(state.borrow_mut().deref_mut()).render(area, buf),
    }
  }
}
