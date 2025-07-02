use super::*;
use crate::ui::{blk, clear};
use crate::widgets::{Alert, Input, Logs, Statistic};
use crossterm::event::{Event, KeyEvent};
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Wrap};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;

mod impls;

pub enum InOutChangeMode {
  Input,
  Output,
  Email,
  Password,
}

pub struct App {
  // bg task
  #[allow(dead_code)]
  tasks: JoinSet<()>,
  event_watcher: WatchTx<RenderEvent>,
  popup: Option<Popup>,

  // App data
  focus: bool,
  input: Option<PathBuf>,
  output: Option<PathBuf>,
  change_mode: Option<InOutChangeMode>,
  scrols: ScrollStates,
  statistic: Statistic,
  email: Arc<RwLock<Arc<str>>>,
  password: Arc<RwLock<Arc<str>>>,

  pub output_tx: WatchTx<PathBuf>,
  pub input_tx: WatchTx<PathBuf>,
  pub logs: Logs,
}

impl App {
  pub fn subscribe_event(&self) -> WatchRx<RenderEvent> {
    self.event_watcher.subscribe()
  }

  pub async fn handle(&mut self, event: Event) -> RenderEvent {
    if let Event::FocusGained | Event::FocusLost = event {
      return RenderEvent::no_ops();
    } else if let Event::Resize { .. } = event {
      return RenderEvent::render();
    } else if let Some(event) = self.try_handle_popup(&event).await {
      return event;
    }

    match event {
      Event::Key(key) => {
        // First try to handle logs scrolling if no popup is active
        if let Some(handled) = self.logs.handle_key(key).await {
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
      _ => {}
    }

    RenderEvent::no_ops()
  }

  async fn try_handle_popup(&mut self, event: &Event) -> Option<RenderEvent> {
    if let Some(popup) = self.popup.as_mut()
      && let Some(handled) = popup.handle_event(event).await
    {
      if handled.kind.is_handled() {
        match self.popup.take().unwrap() {
          Popup::Input(input) => {
            return match self.change_mode {
              Some(InOutChangeMode::Email) => {
                *self.email.write().await = Arc::from(input.value());
                Some(RenderEvent::render())
              }
              Some(InOutChangeMode::Password) => {
                *self.password.write().await = Arc::from(input.value());
                Some(RenderEvent::render())
              }
              _ => Some(RenderEvent::no_ops()),
            };
          }
          Popup::Confirmation(_) => todo!(),
          Popup::Warning(_) => todo!(),
          Popup::Alert(_) => return Some(RenderEvent::render()),
          Popup::FileExplorer(state) => {
            return match self.change_mode {
              Some(InOutChangeMode::Input) => Some(self.change_input(state.take().get().await).await),
              Some(InOutChangeMode::Output) => Some(self.change_output(state.take().get().await)),
              _ => Some(RenderEvent::no_ops()),
            };
          }
        }
      } else if handled.kind.is_canceled() {
        self.popup.take();

        return Some(RenderEvent::render());
      } else {
        return Some(handled);
      }
    }

    None
  }

  fn change_output(&mut self, input: Option<PathBuf>) -> RenderEvent {
    self.output = input;
    self.output_tx.send_modify(|current| *current = self.output.clone().unwrap());

    RenderEvent::render()
  }

  async fn change_input(&mut self, input: Option<PathBuf>) -> RenderEvent {
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

  async fn handle_key(&mut self, key: KeyEvent) -> Option<RenderEvent> {
    match key {
      keys!(Char('e'), NONE, Press) => {
        self.popup = Some(Popup::Input(Input::new(
          "Add/Change email".to_owned(),
          self.email.read().await.to_string(),
        )));
        self.change_mode = Some(InOutChangeMode::Email);
        Some(RenderEvent::render())
      }
      keys!(Char('p'), NONE, Press) => {
        self.popup = Some(Popup::Input(Input::new(
          "Add/Change Password".to_owned(),
          "*".repeat(self.password.read().await.len()),
        )));
        self.change_mode = Some(InOutChangeMode::Password);
        Some(RenderEvent::render())
      }
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

  fn draw_input_widget(&self) -> impl Widget {
    let block = blk()
      .title_top(Line::raw(" Input File: ").fg(Color::White))
      .title_alignment(Alignment::Left)
      .fg(Color::Yellow);
    let input_value = if let Some(ref path) = self.input {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(Line::raw(input_value).fg(Color::LightCyan))
      .block(block)
      .wrap(Wrap { trim: true })
      .scroll(self.scrols.input_widget)
  }

  fn draw_output_widget(&self) -> impl Widget {
    let block = blk()
      .title_top(Line::raw(" Output File: ").fg(Color::White))
      .title_alignment(Alignment::Left)
      .fg(Color::Yellow);

    let input_value = if let Some(ref path) = self.output {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(Line::raw(input_value).fg(Color::Blue))
      .block(block)
      .wrap(Wrap { trim: true })
      .scroll(self.scrols.output_widget)
  }

  fn draw_email_widget(&self) -> impl Widget {
    let block = blk()
      .fg(Color::Red)
      .title_top(Line::styled(" ✉️ Email ", Style::new()).left_aligned().fg(Color::White));

    Paragraph::new(Line::raw(self.email.blocking_read().to_string()).fg(Color::DarkGray)).block(block)
  }

  fn draw_password_widget(&self) -> impl Widget {
    let pwd_len = self.password.blocking_read().len();

    let block = blk()
      .fg(Color::Red)
      .title_top(Line::styled(" 🔑 Password", Style::new()).left_aligned().fg(Color::White));

    Paragraph::new(Line::raw("*".repeat(pwd_len)).fg(Color::DarkGray)).block(block)
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
    let controls = Layout::horizontal([Constraint::Percentage(60), Constraint::Fill(1)]).split(controls);
    let [input_area, output_area, email_area, password_area] = Layout::vertical([Constraint::Length(3); 4]).areas(controls[0]);

    tokio::task::block_in_place(|| {
      clear(input_area, buf);
      self.draw_input_widget().render(input_area, buf);
      clear(output_area, buf);
      self.draw_output_widget().render(output_area, buf);
      clear(email_area, buf);
      self.draw_email_widget().render(email_area, buf);
      clear(password_area, buf);
      self.draw_password_widget().render(password_area, buf);

      // Render statistic widget in the right column
      self.statistic.render(controls[1], buf);

      self.logs.render(activity, buf);

      if let Some(ref popup) = self.popup {
        popup.render(area, buf)
      }
    });
  }
}
