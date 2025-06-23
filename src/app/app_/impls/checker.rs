use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use helper::UnhandledEvent;
use tokio::task::JoinHandle;

use crate::app::handler::BucketStatus;
use crate::app::{MpscRx, MpscTx, WatchTx};
use crate::widgets::{Log, Logs};

#[derive(Default, Debug)]
pub struct TaskQueue(VecDeque<JoinHandle<()>>);

impl TaskQueue {
  pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
    let task = tokio::task::spawn_local(future);
    self.push_back(task);
  }

  pub fn maintain(&mut self) {
    self.retain(|task| !task.is_finished());
  }
}

impl Deref for TaskQueue {
  type Target = VecDeque<JoinHandle<()>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl DerefMut for TaskQueue {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub async fn line_checker(mut line_rx: MpscRx<Arc<str>>, bucket_tx: MpscTx<BucketStatus>, event: WatchTx<UnhandledEvent>, logs: Logs) {
  let mut tasks = TaskQueue::default();

  while let Some(line) = line_rx.recv().await {
    tasks.maintain();
    tasks.spawn(check(line, bucket_tx.clone(), event.clone(), logs.clone()));
  }

  futures::future::join_all(tasks.0).await;
}

pub async fn check(domain: Arc<str>, bucket_tx: MpscTx<BucketStatus>, event: WatchTx<UnhandledEvent>, logs: Logs) {
  match BucketStatus::new(domain).await {
    Ok(status) => match bucket_tx.send(status.clone()).await {
      Ok(_) => {
        logs.add(Log::bucket(status)).await;
        event.send_modify(|e| *e = UnhandledEvent::render());
      }
      Err(err) => {
        logs.add(Log::error(err)).await;
        event.send_modify(|e| *e = UnhandledEvent::render());
      }
    },
    Err(err) => {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = UnhandledEvent::render());
    }
  }
}
