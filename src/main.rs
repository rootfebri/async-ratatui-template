use std::io::{Result, stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, MouseEvent};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use helper::{PollEvent, UnhandledEvent, keys};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame, Terminal};
use tokio::select;
use tokio::task::block_in_place;

use crate::app::App;

pub type Area = Rect;
pub mod app;
pub mod areas;
pub mod ui;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
  let backend = CrosstermBackend::new(stdout());
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;
  enable_raw_mode()?;
  execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;

  let mut status: Result<()> = Ok(());
  let mut event = PollEvent::default();
  let mut app = App::default();
  let mut app_event = app.subscribe_event();

  // Initiate first render
  terminal.draw(|frame| frame.render_widget(&app, frame.area()))?;

  while status.is_ok() && !app.exit() {
    let handled = select! {
      _ = app_event.changed() =>  app_event.borrow_and_update().clone(),
      poll = event.fuse_read() => match poll {
        Ok(event) => app.handle(event).await,
        Err(error) => {
          status = Err(error);
          continue;
        },
      },
    };

    if handled.kind.is_render() {
      terminal.draw(|frame| frame.render_widget(&app, frame.area()))?;
    } else if handled.kind.is_handled() {
      break;
    }
  }

  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
  terminal.show_cursor()?;

  status
}

#[macro_export]
macro_rules! never {
  () => {{
    use ::tokio::time;

    loop {
      time::sleep(time::Duration::from_secs(10)).await
    }
  }};
}

pub fn mouse_area(mouse_event: &MouseEvent) -> Area {
  Area::new(mouse_event.column, mouse_event.row, 1, 1)
}

#[allow(unused)]
async fn test_widget(
  terminal: &mut DefaultTerminal,
  poll_event: &mut PollEvent,
  mut drawer: impl FnMut(&mut Frame),
  mut event_handler: impl AsyncFnMut(Event) -> UnhandledEvent,
) -> Result<()> {
  let mut run = true;

  while run {
    select! {
      event = poll_event.read() => {
        let event = event?;
        if let Event::Key(keys!(Esc, NONE, Press)) = event {
          run = false;
          continue;
        } else {
          #[allow(clippy::needless_borrow)]
          let out: UnhandledEvent = (&mut event_handler)(event).await;
          if out.kind.is_no_ops() {
            continue
          }
        }
      }
    }

    block_in_place(|| terminal.draw(&mut drawer))?;
  }

  Ok(())
}
