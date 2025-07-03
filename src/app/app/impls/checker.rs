use crate::app::handler::sectrails::SecTrailClient;
use crate::app::handler::sectrails::jsons::Record;
use crate::app::{MpscRx, MpscTx, SYNC_STATE, WatchTx};
use crate::args::AppArgs;
use crate::wait_process;
use crate::widgets::{Log, Logs, Statistic};
use clap::Parser;
use helper::RenderEvent;
use ratatui::prelude::{Span, Stylize};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::Instant;

pub async fn line_checker(mut line_rx: MpscRx<Arc<str>>, bucket_tx: MpscTx<Record>, event: WatchTx<RenderEvent>, logs: Logs, statistic: Statistic) {
  let fps = AppArgs::parse().fps.max(5);
  let mut sectrail = SecTrailClient::new();

  while let Some(line) = line_rx.recv().await {
    wait_process!();
    select! {
      _ = async {
        while SYNC_STATE.is_processing() {
          tokio::time::sleep(Duration::from_millis(1000 / fps as u64)).await;
        }
      } => {}
      _ = check(&mut sectrail, &line, &bucket_tx, &event, &logs, &statistic) => {}
    }
  }
}

pub async fn check(
  sectrail: &mut SecTrailClient,
  line: &str,
  recorder: &MpscTx<Record>,
  event: &WatchTx<RenderEvent>,
  logs: &Logs,
  statistic: &Statistic,
) {
  logs.add(Log::info(format!("Entering new SecTrail Sequence: Data = {line}"))).await;

  let mut total_records = 0;
  sectrail.new_sequence(line);

  loop {
    let rpm = Instant::now();
    match sectrail.get().await {
      Ok(response) if response.as_records().is_empty() => break,
      Ok(response) => {
        let records = response.into_records();
        total_records += records.len();
        for record in records {
          logs.add(Log::record(record.clone())).await;
          recorder.send(record).await.unwrap();
        }
      }
      Err(err) => {
        let error_span = Span::raw(err.to_string()).red();
        logs.add(Log::error(err)).await;
        statistic.increment_errors();
        event.send(RenderEvent::error(error_span)).unwrap();
      }
    }
    tokio::time::sleep_until(rpm + Duration::from_secs(6)).await;
  }

  logs.info(format!("Added {total_records} records from `{line}`")).await;
  statistic.increment();
  statistic.add_bytes_processed(line.len());
  statistic.update_rate();
}
