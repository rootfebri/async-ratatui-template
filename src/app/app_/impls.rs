use helper::RenderEvent;
use std::time::Duration;

use super::*;
use crate::ARGS;
use crate::app::app_::impls::checker::line_checker;
use crate::app::app_::impls::reader::input_reader;
use crate::app::app_::impls::writer::output_writer;
use crate::args::AppArgs;
use crate::widgets::Statistic;

impl Default for App {
  fn default() -> Self {
    let logs: Logs = Default::default();
    let mut tasks = JoinSet::<()>::new();
    let event_watcher: WatchTx<RenderEvent> = WatchTx::default();
    let (line_tx, line_rx) = mpsc::channel(1);
    let (recorder, records) = mpsc::channel(1024);
    let statistic = Statistic::default();

    tasks.spawn(input_reader(line_tx, event_watcher.clone(), logs.clone(), statistic.clone()));
    tasks.spawn(line_checker(line_rx, recorder, event_watcher.clone(), logs.clone(), statistic.clone()));
    tasks.spawn(output_writer(records, event_watcher.clone(), logs.clone()));

    Self {
      popup: None,
      change_mode: None,
      tasks,
      event_watcher,
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

async fn change_listener(ops: impl Fn(&AppArgs) -> bool) {
  loop {
    let args = ARGS.read().await;
    let fps = args.fps;
    if ops(&args) {
      break;
    } else {
      drop(args);
      tokio::time::sleep(Duration::from_millis(fps.max(5) as u64)).await
    }
  }
}
