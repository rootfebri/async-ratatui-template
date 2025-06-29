use std::path::PathBuf;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::widgets::{Paragraph, Wrap};
use tokio::task::JoinSet;

use super::*;
use crate::ui::blk;
use crate::widgets::{Alert, Logs, Statistic};

mod impls;

pub enum InOutChangeMode {
  Input,
  Output,
}

pub struct App {
  popup: Option<Popup>,

  // App data
  input: Option<PathBuf>,
  output: Option<PathBuf>,
  change_mode: Option<InOutChangeMode>,

  // bg task
  #[allow(dead_code)]
  tasks: JoinSet<()>,
  event_watcher: WatchTx<RenderEvent>,
  pub output_tx: WatchTx<PathBuf>,
  pub input_tx: WatchTx<PathBuf>,
  focus: bool,
  scrols: ScrollStates,
  pub(crate) logs: Logs,
  statistic: Statistic,
}

impl App {
  pub fn subscribe_event(&self) -> WatchRx<RenderEvent> {
    self.event_watcher.subscribe()
  }

  pub async fn handle_mouse(&mut self, mouse: MouseEvent) -> RenderEvent {
    if let Some(handle) = self.logs.handle_mouse(mouse).await {
      return handle;
    }

    RenderEvent::no_ops()
  }

  pub fn change_output(&mut self, input: Option<PathBuf>) -> RenderEvent {
    self.output = input;
    self.output_tx.send_modify(|current| *current = self.output.clone().unwrap());

    RenderEvent::render()
  }

  pub async fn change_input(&mut self, input: Option<PathBuf>) -> RenderEvent {
    if let Some(path) = input {
      if !path.exists() {
        let alert = ["File does not exists.".to_string(), format!("File {}.", path.display())];
        let alert = Alert::new("Invalid Input", alert.into_iter());
        self.popup = Some(Popup::Alert(alert));
        return RenderEvent::render();
      }

      if !path.is_file() {
        let alert = ["Input is not a file.".to_string(), format!("File {}.", path.display())];
        let alert = Alert::new("Invalid Input", alert.into_iter());
        self.popup = Some(Popup::Alert(alert));

        return RenderEvent::render();
      }

      self.input_tx.send_modify(|watch_path| *watch_path = path.clone());
      self.input = Some(path);

      return RenderEvent::render();
    }

    RenderEvent::no_ops()
  }

  pub async fn handle_key(&mut self, key: KeyEvent) -> Option<RenderEvent> {
    match key {
      keys!(Char('i'), NONE, Press) => {
        self.popup = Some(Popup::FileExplorer(Default::default()));
        self.change_mode = Some(InOutChangeMode::Input);
        Some(RenderEvent::render())
      }
      keys!(Char('o'), NONE, Press) => {
        self.popup = Some(Popup::FileExplorer(Default::default()));
        self.change_mode = Some(InOutChangeMode::Output);
        Some(RenderEvent::render())
      }
      keys!(Char('s'), NONE, Press) => {
        SYNC_STATE.process();
        Some(RenderEvent::render())
      }
      keys!(Char('c'), CONTROL, Press) => {
        SYNC_STATE.exit();
        Some(RenderEvent::render())
      }
      keys!(Esc, NONE, Press) => {
        SYNC_STATE.idle();
        Some(RenderEvent::render())
      }
      KeyEvent { .. } => None,
    }
  }

  pub async fn handle(&mut self, event: Event) -> RenderEvent {
    if let Event::FocusGained | Event::FocusLost = event {
      return RenderEvent::no_ops();
    } else if let Event::Resize { .. } = event {
      return RenderEvent::render();
    }

    if let Some(popup) = self.popup.as_mut()
      && let Some(handled) = popup.handle_event(&event).await
    {
      if handled.kind.is_handled() {
        match self.popup.take().unwrap() {
          Popup::Input(_) => todo!(),
          Popup::Confirmation(_) => todo!(),
          Popup::Warning(_) => todo!(),
          Popup::Alert(_) => return RenderEvent::render(),
          Popup::FileExplorer(state) => {
            return match self.change_mode {
              Some(InOutChangeMode::Input) => self.change_input(state.take().get().await).await,
              Some(InOutChangeMode::Output) => self.change_output(state.take().get().await),
              _ => RenderEvent::no_ops(),
            };
          }
        }
      } else if handled.kind.is_canceled() {
        self.popup.take();

        return RenderEvent::render();
      } else {
        return handled;
      }
    }

    match event {
      Event::FocusGained => { /*self.focus = true*/ }
      Event::FocusLost => {
        /*
        self.focus = false;
        return UnhandledEvent::render();
        */
      }
      Event::Key(key) => {
        // First try to handle logs scrolling if no popup is active
        if self.popup.is_none()
          && let Some(handled) = self.logs.handle_key(key).await
        {
          return handled;
        }

        if let Some(unhandled_event) = self.handle_key(key).await {
          return unhandled_event;
        }
      }
      Event::Mouse(mouse) => {
        if let Some(handled) = self.logs.handle_mouse(mouse).await {
          return handled;
        }
      }
      Event::Paste(_) => {}
      Event::Resize(_, _) => return RenderEvent::render(),
    }

    RenderEvent::no_ops()
  }

  fn draw_input_widget(&self) -> impl Widget {
    let block = blk().title_top(" Input File: ").title_alignment(Alignment::Left);
    let input_value = if let Some(ref path) = self.input {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(input_value)
      .block(block)
      .wrap(Wrap { trim: true })
      .scroll(self.scrols.input_widget)
  }

  fn draw_output_widget(&self) -> impl Widget {
    let block = blk().title_top(" Output File: ").title_alignment(Alignment::Left);
    let input_value = if let Some(ref path) = self.output {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(input_value)
      .block(block)
      .wrap(Wrap { trim: true })
      .scroll(self.scrols.output_widget)
  }
}

impl Widget for &App {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    if !self.focus {
      return;
    }

    let [controls, activity] = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]).areas(area);
    let controls = Layout::horizontal([Constraint::Percentage(68), Constraint::Fill(1)]).split(controls);
    let control_chunks = Layout::vertical([Constraint::Length(3), Constraint::Length(3), Constraint::Fill(1)]).split(controls[0]);

    tokio::task::block_in_place(|| {
      self.draw_input_widget().render(control_chunks[0], buf);
      self.draw_output_widget().render(control_chunks[1], buf);

      // Render statistic widget in the right column
      self.statistic.render(controls[1], buf);

      self.logs.render(activity, buf);

      if let Some(ref popup) = self.popup {
        popup.render(area, buf)
      }
    });
  }
}
