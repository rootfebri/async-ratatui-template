use std::sync::Arc;

use helper::RenderEvent;

use crate::app::handler::sectrails::SecTrailClient;
use crate::app::handler::sectrails::jsons::Record;
use crate::app::{MpscRx, MpscTx, WatchTx};
use crate::widgets::{Log, Logs, Statistic};
use crate::{never, wait_process};

pub async fn line_checker(mut line_rx: MpscRx<Arc<str>>, bucket_tx: MpscTx<Record>, event: WatchTx<RenderEvent>, logs: Logs, statistic: Statistic) {
  let mut sectrail = SecTrailClient::new();

  while let Some(line) = line_rx.recv().await {
    wait_process!();
    check(&mut sectrail, &line, &bucket_tx, &event, &logs, &statistic).await;
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
  logs.add(Log::info(format!("Entering new SecTrail Sequence => Data = {line}"))).await;

  let mut total_records = 0;
  sectrail.new_sequence(line);
  loop {
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
        logs.add(Log::error(err)).await;
        event.send(RenderEvent::handled()).unwrap();

        never!();
      }
    }
  }

  logs.info(format!("Added {total_records} records from `{line}`")).await;
  statistic.increment();
}
