use crossterm::event::KeyEvent;
use futures::FutureExt;
use futures::future::BoxFuture;
use helper::{RenderEvent, keys};

use super::*;
use crate::app::App;
use crate::ui::{blk, clear};

type Action = Box<dyn for<'a> FnOnce(&'a mut App) -> BoxFuture<'a, RenderEvent>>;

pub struct AppAction(Action);

#[macro_export]
macro_rules! app_action {
  (async |$var:ident| $body:expr) => {{
    use futures::FutureExt;
    $crate::widgets::AppAction::new(|$var: &mut $crate::App| async { $body }.boxed())
  }};
}

impl Default for AppAction {
  fn default() -> Self {
    Self(Box::new(|_: &mut App| async { RenderEvent::no_ops() }.boxed()))
  }
}

impl AppAction {
  pub fn new(f: impl for<'a> FnOnce(&'a mut App) -> BoxFuture<'a, RenderEvent> + 'static) -> Self {
    Self(Box::from(f))
  }
}

pub struct Confirmation {
  title: String,
  message: String,
  value: Option<bool>,
  on_submit: Option<AppAction>,
  on_cancel: Option<AppAction>,
}

impl Confirmation {
  pub fn handle_key(&mut self, key: KeyEvent) -> Option<RenderEvent> {
    match key {
      keys!(Char('y'), NONE, Press) | keys!(Enter, NONE, Press) => {
        self.value = Some(true);
        Some(RenderEvent::handled())
      }
      keys!(Char('n'), NONE, Press) | keys!(Esc, NONE, Press) => {
        self.value = Some(false);
        Some(RenderEvent::canceled())
      }
      _ => None,
    }
  }

  pub fn new(message: String) -> Self {
    Self {
      title: "Confirmation".to_string(),
      message,
      value: None,
      on_submit: None,
      on_cancel: None,
    }
  }

  pub fn on_submit(mut self, on_submit: AppAction) -> Self {
    self.on_submit = Some(on_submit);
    self
  }

  pub fn on_cancel(mut self, on_cancel: AppAction) -> Self {
    self.on_cancel = Some(on_cancel);
    self
  }

  pub fn title(mut self, title: String) -> Self {
    self.title = title;
    self
  }

  pub async fn call_submit(self, app: &mut App) -> Option<RenderEvent> {
    let call_submit = self.on_submit?;
    Some(call_submit.0(app).await)
  }

  pub async fn call_cancel(self, app: &mut App) -> Option<RenderEvent> {
    let call_cancel = self.on_cancel?;
    Some(call_cancel.0(app).await)
  }
}

impl Widget for &Confirmation {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let title_top = Line::styled(self.title.as_str(), Color::Yellow).centered();
    let title_bottom = [
      Span::raw("[").white(),
      Span::raw("<Y/Enter>").green(),
      Span::raw("Confirm").white(),
      Span::raw("]").white(),
      Span::raw(" ").white(),
      Span::raw("[").white(),
      Span::raw("<N/Esc>").red(),
      Span::raw("Cancel").white(),
      Span::raw("]").white(),
    ]
    .into_iter()
    .collect::<Line>()
    .centered();

    let block = blk().fg(Color::LightYellow).title_top(title_top).title_bottom(title_bottom);
    let message = Line::from(self.message.as_str()).white().centered();

    clear(area, buf);
    (&block).render(area, buf);
    message.render(block.inner(area), buf);
  }
}
