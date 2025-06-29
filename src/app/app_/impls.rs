use helper::UnhandledEvent;

use super::*;
use crate::app::app_::impls::checker::line_checker;
use crate::app::app_::impls::reader::input_reader;
use crate::app::app_::impls::writer::output_writer;
use crate::widgets::Statistic;

impl Default for App {
  fn default() -> Self {
    let logs: Logs = Default::default();
    let mut tasks = JoinSet::<()>::new();
    let event_watcher: WatchTx<UnhandledEvent> = WatchTx::default();
    let (line_tx @ MpscTx { .. }, line_rx @ MpscRx { .. }) = mpsc::channel(1);
    let (bucket_tx @ MpscTx { .. }, bucket_rx @ MpscRx { .. }) = mpsc::channel(1024);
    let output_tx = WatchTx::new(Default::default());
    let input_tx = WatchTx::new(Default::default());
    let state_tx = WatchTx::new(State::Iddling);
    let statistic = Statistic::new("Processing Domains");

    tasks.spawn(input_reader(
      line_tx,
      input_tx.subscribe(),
      event_watcher.clone(),
      logs.clone(),
      statistic.clone(),
      state_tx.subscribe(),
    ));
    tasks.spawn(line_checker(
      line_rx,
      bucket_tx,
      event_watcher.clone(),
      logs.clone(),
      statistic.clone(),
      state_tx.subscribe(),
    ));
    tasks.spawn(output_writer(
      bucket_rx,
      output_tx.subscribe(),
      event_watcher.clone(),
      logs.clone(),
      state_tx.subscribe(),
    ));

    Self {
      state: State::Iddling,
      popup: None,
      input: None,
      output: None,
      change_mode: None,
      tasks,
      event_watcher,
      output_tx,
      input_tx,
      state_tx,
      focus: true,
      scrols: ScrollStates::default(),
      logs,
      statistic,
    }
  }
}

mod checker;
mod reader;
mod writer;
