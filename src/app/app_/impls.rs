use clap::Parser;
use helper::RenderEvent;

use super::*;
use crate::app::app_::impls::checker::line_checker;
use crate::app::app_::impls::reader::input_reader;
use crate::app::app_::impls::writer::output_writer;
use crate::args::AppArgs;
use crate::widgets::Statistic;
use crate::widgets::pulse::{PulseFps, PulseState};

impl Default for App {
  fn default() -> Self {
    let AppArgs {
      email,
      input,
      output,
      password,
      fps,
      ..
    } = AppArgs::parse();
    let email = Arc::new(RwLock::new(Arc::from(email.unwrap_or_default())));
    let password = Arc::new(RwLock::new(Arc::from(password.unwrap_or_default())));

    let logs: Logs = Default::default();
    let mut tasks = JoinSet::<()>::new();
    let event_watcher: WatchTx<RenderEvent> = WatchTx::default();
    let (line_tx, line_rx) = mpsc::channel(1);
    let (recorder, records) = mpsc::channel(1024);
    let output_tx = WatchTx::new(output.clone().unwrap_or_default());
    let input_tx = WatchTx::new(input.clone().unwrap_or_default());
    let statistic = Statistic::default().pulse_state(PulseState::new(PulseFps::from(fps)));

    tasks.spawn(input_reader(
      line_tx,
      input_tx.subscribe(),
      event_watcher.clone(),
      logs.clone(),
      statistic.clone(),
    ));

    tasks.spawn(line_checker(
      line_rx,
      recorder,
      event_watcher.clone(),
      logs.clone(),
      statistic.clone(),
      email.clone(),
      password.clone(),
    ));
    tasks.spawn(output_writer(records, output_tx.subscribe(), event_watcher.clone(), logs.clone()));

    Self {
      popup: None,
      input,
      output,
      change_mode: None,
      tasks,
      event_watcher,
      output_tx,
      input_tx,
      focus: true,
      scrols: ScrollStates::default(),
      logs,
      statistic,
      email,
      password,
    }
  }
}

mod checker;
mod reader;
mod writer;
