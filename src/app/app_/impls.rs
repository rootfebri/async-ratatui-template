use helper::RenderEvent;

use super::*;
use crate::app::app_::impls::checker::line_checker;
use crate::app::app_::impls::reader::input_reader;
use crate::app::app_::impls::writer::output_writer;
use crate::widgets::Statistic;

impl Default for App {
  fn default() -> Self {
    let logs: Logs = Default::default();
    let mut tasks = JoinSet::<()>::new();
    let event_watcher: WatchTx<RenderEvent> = WatchTx::default();
    let (line_tx, line_rx) = mpsc::channel(1);
    let (recorder, records) = mpsc::channel(1024);
    let output_tx = WatchTx::new(Default::default());
    let input_tx = WatchTx::new(Default::default());
    let statistic = Statistic::new("Processing Domains");

    tasks.spawn(input_reader(
      line_tx,
      input_tx.subscribe(),
      event_watcher.clone(),
      logs.clone(),
      statistic.clone(),
    ));

    tasks.spawn(line_checker(line_rx, recorder, event_watcher.clone(), logs.clone(), statistic.clone()));
    tasks.spawn(output_writer(records, output_tx.subscribe(), event_watcher.clone(), logs.clone()));

    Self {
      popup: None,
      input: None,
      output: None,
      change_mode: None,
      tasks,
      event_watcher,
      output_tx,
      input_tx,
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
