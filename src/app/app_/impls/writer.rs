use std::path::PathBuf;

use helper::UnhandledEvent;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::watch::Sender;
use tokio::{fs, select};

use crate::app::handler::BucketStatus;
use crate::app::{MpscRx, WatchRx};
use crate::never;
use crate::widgets::{Log, Logs};

pub async fn output_writer(mut bucket_rx: MpscRx<BucketStatus>, mut output_rx: WatchRx<PathBuf>, event: Sender<UnhandledEvent>, logs: Logs) {
  let mut output = output_rx.borrow_and_update().clone();
  loop {
    select! {
      new_output = output_rx.wait_for(|path| *path != output) => output = new_output.unwrap().clone(),
      _ = writer(&mut bucket_rx, &output, &event, logs.clone()) => continue
    }
  }
}

pub async fn writer(rx: &mut MpscRx<BucketStatus>, output: &PathBuf, event: &Sender<UnhandledEvent>, logs: Logs) {
  let info = format!("Writer working on `{}`", output.display());
  logs.add(Log::info(info)).await;

  let file = match fs::File::options().create(true).append(true).open(output).await {
    Ok(file) => {
      logs.add(Log::info(format!("Opened file: {}", output.display()))).await;
      file
    }
    Err(err) => {
      logs.add(Log::error(err)).await;
      never!()
    }
  };

  let mut writer = BufWriter::new(file);
  while let Some(bucket) = rx.recv().await.map(String::from) {
    if let Err(err) = writer.write_all(bucket.as_bytes()).await {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = UnhandledEvent::render());
    } else {
      logs.add(Log::info(format!("Wrote bucket: {bucket}"))).await;
    }

    if let Err(err) = writer.write_all(b"\n").await {
      event.send_modify(|e| *e = UnhandledEvent::render());
      logs.add(Log::error(err)).await;
    }
    if let Err(err) = writer.flush().await {
      event.send_modify(|e| *e = UnhandledEvent::render());
      logs.add(Log::error(err)).await;
    }
  }

  never!()
}
