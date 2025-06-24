use std::path::PathBuf;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use helper::{UnhandledEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::widgets::{Paragraph, Wrap};
use tokio::task::JoinSet;

use super::*;
use crate::ui::blk;
use crate::widgets::{Alert, Input, Logs, Statistic};

mod impls;

pub enum InOutChangeMode {
  Input,
  Output,
}

pub struct App {
  state: State,
  popup: Option<Popup>,

  // App data
  input: Option<PathBuf>,
  output: Option<PathBuf>,
  change_mode: Option<InOutChangeMode>,

  // bg task
  #[allow(dead_code)]
  tasks: JoinSet<()>,
  event_watcher: WatchTx<UnhandledEvent>,
  pub output_tx: WatchTx<PathBuf>,
  pub input_tx: WatchTx<PathBuf>,
  pub state_tx: WatchTx<State>,
  focus: bool,
  scrols: ScrollStates,
  logs: Logs,
  statistic: Statistic,
}

impl App {
  pub fn exit(&self) -> bool {
    self.state == State::Exit
  }

  pub fn subscribe_event(&self) -> WatchRx<UnhandledEvent> {
    self.event_watcher.subscribe()
  }

  pub async fn handle_mouse(&mut self, mouse: MouseEvent) -> UnhandledEvent {
    if let Some(handle) = self.logs.handle_mouse(mouse).await {
      return handle;
    }

    UnhandledEvent::no_ops()
  }

  pub fn change_output(&mut self, input: Input) -> UnhandledEvent {
    let output_file = PathBuf::from(input.value());
    self.output = Some(output_file);
    self.output_tx.send_modify(|current| *current = self.output.clone().unwrap());

    UnhandledEvent::render()
  }

  pub async fn change_input(&mut self, input: Input) -> UnhandledEvent {
    let input_file = PathBuf::from(input.value());
    if !input_file.exists() {
      let alert = ["File does not exists.".to_string(), format!("File {}.", input_file.display())];
      let alert = Alert::new("Invalid Input", alert.into_iter());
      self.popup = Some(Popup::Alert(alert));
      return UnhandledEvent::render();
    }
    if !input_file.is_file() {
      let alert = ["Input is not a file.".to_string(), format!("File {}.", input_file.display())];
      let alert = Alert::new("Invalid Input", alert.into_iter());
      self.popup = Some(Popup::Alert(alert));
      return UnhandledEvent::render();
    }

    self.input_tx.send_modify(|watch_path| *watch_path = input_file.clone());
    self.input = Some(input_file);

    UnhandledEvent::render()
  }

  pub async fn handle_key(&mut self, key: KeyEvent) -> Option<UnhandledEvent> {
    match key {
      keys!(Char('i'), NONE, Press) => {
        let input = Input::new(" Enter full/relative path to file input: ".to_string(), "Start typing...".to_string());
        let popup = Popup::Input(input);
        self.popup = Some(popup);
        self.change_mode = Some(InOutChangeMode::Input);
        Some(UnhandledEvent::render())
      }
      keys!(Char('o'), NONE, Press) => {
        let input = Input::new(" Enter full/relative path to file output: ".to_string(), "Start typing...".to_string());
        let popup = Popup::Input(input);
        self.popup = Some(popup);
        self.change_mode = Some(InOutChangeMode::Output);
        Some(UnhandledEvent::render())
      }
      keys!(Char('s'), NONE, Press) => {
        self.state = State::Processing;
        self.state_tx.send_modify(|current| *current = self.state);
        Some(UnhandledEvent::render())
      }
      keys!(Char('c'), CONTROL, Press) => {
        self.state = State::Exit;
        self.state_tx.send_modify(|current| *current = self.state);
        Some(UnhandledEvent::render())
      }
      keys!(Esc, NONE, Press) => {
        self.state = State::Iddling;
        self.state_tx.send_modify(|current| *current = self.state);
        Some(UnhandledEvent::render())
      }
      KeyEvent { .. } => None,
    }
  }

  pub async fn handle(&mut self, event: Event) -> UnhandledEvent {
    if let Some(popup) = self.popup.as_mut()
      && let Some(handled) = popup.handle_event(&event)
    {
      if handled.kind.is_handled() {
        match self.popup.take().unwrap() {
          Popup::Input(input) => match self.change_mode {
            None => {}
            Some(ref mode) => {
              return match mode {
                InOutChangeMode::Input => self.change_input(input).await,
                InOutChangeMode::Output => self.change_output(input),
              };
            }
          },
          Popup::Confirmation(_) => todo!(),
          Popup::Warning(_) => todo!(),
          Popup::Alert(_) => return UnhandledEvent::render(),
        }
      } else if handled.kind.is_canceled() {
        self.popup.take();

        return UnhandledEvent::render();
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
      Event::Resize(_, _) => return UnhandledEvent::render(),
    }

    UnhandledEvent::no_ops()
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
