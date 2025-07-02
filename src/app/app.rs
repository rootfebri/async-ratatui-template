use crossterm::event::{Event, KeyEvent};
use helper::{RenderEvent, keys};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::style::Color;
use ratatui::widgets::{Paragraph, Wrap};
use tokio::task::JoinSet;

use super::*;
use crate::ui::{blk, clear};
use crate::widgets::{Confirmation, Input, Log, Logs, Statistic, line_with_hotkey};
use crate::{ARGS, app_action};

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
  change_mode: Option<InOutChangeMode>,
  statistic: Statistic,

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

        if let Some(render_event) = self.handle_key(key).await {
          return render_event;
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
    let popup = self.popup.as_mut()?;
    let handled = popup.handle_event(event).await?;

    if handled.kind.is_handled() {
      match self.popup.take()? {
        Popup::Input(input) => match self.change_mode {
          Some(InOutChangeMode::Email) => {
            ARGS.write().await.email = Some(input.value());
            Some(RenderEvent::render())
          }
          Some(InOutChangeMode::Password) => {
            ARGS.write().await.password = Some(input.value());
            Some(RenderEvent::render())
          }
          _ => Some(RenderEvent::no_ops()),
        },
        Popup::Confirmation(confirmation) => confirmation.call_submit(self).await,
        Popup::Warning(_) => todo!(),
        Popup::Alert(_) => Some(RenderEvent::render()),
        Popup::FileExplorer(state) => match self.change_mode {
          Some(InOutChangeMode::Input) => {
            ARGS.write().await.input = state.take().get().await;
            Some(RenderEvent::render())
          }
          Some(InOutChangeMode::Output) => {
            ARGS.write().await.output = state.take().get().await;
            Some(RenderEvent::render())
          }
          _ => Some(RenderEvent::no_ops()),
        },
      }
    } else if handled.kind.is_canceled() {
      match self.popup.take()? {
        Popup::Confirmation(modal) => modal.call_cancel(self).await,
        _ => Some(RenderEvent::render()),
      }
    } else {
      Some(handled)
    }
  }

  async fn handle_key(&mut self, key: KeyEvent) -> Option<RenderEvent> {
    match key {
      keys!(Char('e'), NONE, Press) => {
        self.popup = Some(Popup::Input(Input::new(
          "Add/Change email".to_owned(),
          ARGS.read().await.email.clone().unwrap_or_default(),
        )));
        self.change_mode = Some(InOutChangeMode::Email);
        Some(RenderEvent::render())
      }
      keys!(Char('p'), NONE, Press) => {
        self.popup = Some(Popup::Input(Input::new(
          "Add/Change Password".to_owned(),
          "*".repeat(ARGS.read().await.password.as_ref().map(String::len).unwrap_or_default()),
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
      keys!(Char('s'), CONTROL, Press) => {
        SYNC_STATE.process();
        Some(RenderEvent::render())
      }
      keys!(Char('c'), CONTROL, Press) => {
        if SYNC_STATE.is_processing() {
          let on_cancel = app_action!(async |_app| RenderEvent::render());
          let on_submit = app_action!(async |app| {
            app.logs.add(Log::warn("Force exiting...")).await;
            SYNC_STATE.exit();
            RenderEvent::render()
          });

          let state = Confirmation::new("There is still a background process...".to_string())
            .title(String::from("Are you sure?"))
            .on_cancel(on_cancel)
            .on_submit(on_submit);
          self.popup = Some(Popup::Confirmation(state));
        } else {
          SYNC_STATE.exit();
        }

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
    let label: Line = line_with_hotkey("Input File:", true, Color::Red, Color::White);
    let block = blk().title_top(label.fg(Color::White)).title_alignment(Alignment::Left).fg(Color::Yellow);
    let input_value = if let Some(ref path) = ARGS.blocking_read().input {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(Line::raw(input_value).fg(Color::LightCyan))
      .block(block)
      .wrap(Wrap { trim: true })
  }

  fn draw_output_widget(&self) -> impl Widget {
    let label: Line = line_with_hotkey("Output File:", true, Color::Red, Color::White);
    let block = blk().title_top(label).title_alignment(Alignment::Left).fg(Color::Yellow);

    let input_value = if let Some(ref path) = ARGS.blocking_read().output {
      path.display().to_string()
    } else {
      String::from("None")
    };

    Paragraph::new(Line::raw(input_value).fg(Color::Blue))
      .block(block)
      .wrap(Wrap { trim: true })
  }

  fn draw_email_widget(&self) -> impl Widget {
    let label = line_with_hotkey("Email 📧", true, Color::Red, Color::White);
    let block = blk().fg(Color::Red).title_top(label.left_aligned().fg(Color::White));

    Paragraph::new(Line::raw(ARGS.blocking_read().email.clone().unwrap_or_default()).fg(Color::DarkGray)).block(block)
  }

  fn draw_password_widget(&self) -> impl Widget {
    let label = line_with_hotkey("Password 🔑", true, Color::Red, Color::White);
    let pwd_len = ARGS.blocking_read().password.as_ref().map(String::len).unwrap_or_default();
    let block = blk().fg(Color::Red).title_top(label.left_aligned().fg(Color::White));

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
    let controls = Layout::horizontal([Constraint::Min(3 * 4), Constraint::Fill(1)]).split(controls);
    let [input_area, output_area, email_area, password_area] = Layout::vertical([Constraint::Max(3); 4]).areas(controls[0]);

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
