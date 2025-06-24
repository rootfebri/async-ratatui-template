use std::sync::Arc;
use std::time::Duration;

use helper::UnhandledEvent;
use tokio::time::sleep;

use crate::app::handler::BucketStatus;
use crate::app::{MpscRx, MpscTx, State, WatchRx, WatchTx};
use crate::widgets::{Log, Logs, Statistic};

pub async fn line_checker(
  mut line_rx: MpscRx<Arc<str>>,
  bucket_tx: MpscTx<BucketStatus>,
  event: WatchTx<UnhandledEvent>,
  logs: Logs,
  statistic: Statistic,
  state_watcher: WatchRx<State>,
) {
  while let Some(line) = line_rx.recv().await {
    while !state_watcher.borrow().is_processing() {
      sleep(Duration::from_millis(16)).await
    }
    check(line, bucket_tx.clone(), event.clone(), logs.clone(), statistic.clone()).await;
  }
}

pub async fn check(domain: Arc<str>, bucket_tx: MpscTx<BucketStatus>, event: WatchTx<UnhandledEvent>, logs: Logs, statistic: Statistic) {
  match BucketStatus::new(domain).await {
    Ok(status) => match bucket_tx.send(status.clone()).await {
      Ok(_) => {
        logs.add(Log::bucket(status)).await;
        statistic.increment();
        event.send_modify(|e| *e = UnhandledEvent::render());
      }
      Err(err) => {
        logs.add(Log::error(err)).await;
        statistic.increment();
        event.send_modify(|e| *e = UnhandledEvent::render());
      }
    },
    Err(err) => {
      logs.add(Log::error(err)).await;
      statistic.increment();
      event.send_modify(|e| *e = UnhandledEvent::render());
    }
  }
}
