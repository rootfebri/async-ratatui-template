use std::path::PathBuf;

use helper::RenderEvent;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{fs, select};

use crate::app::handler::sectrails::jsons::Record;
use crate::app::{MpscRx, WatchRx, WatchTx};
use crate::widgets::{Log, Logs};
use crate::{never, wait_process};

pub async fn output_writer(mut bucket_rx: MpscRx<Record>, mut output_rx: WatchRx<PathBuf>, event: WatchTx<RenderEvent>, logs: Logs) {
  let mut output = output_rx.borrow_and_update().clone();
  loop {
    select! {
      new_output = output_rx.wait_for(|path| *path != output) => output = new_output.unwrap().clone(),
      _ = writer(&mut bucket_rx, &output, &event, logs.clone()) => continue
    }
  }
}

pub async fn writer(rx: &mut MpscRx<Record>, output: &PathBuf, event: &WatchTx<RenderEvent>, logs: Logs) {
  wait_process!();
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

  let mut total_saved: usize = 0;
  let mut writer = BufWriter::new(file);
  while let Some(bucket) = rx.recv().await {
    if let Err(err) = writer.write_all(bucket.as_csv().as_bytes()).await {
      logs.add(Log::error(err)).await;
      event.send_modify(RenderEvent::as_handled);

      total_saved += 1;
    }

    if let Err(err) = writer.write_all(b"\n").await {
      event.send_modify(RenderEvent::as_handled);
      logs.add(Log::error(err)).await;
    }
    if let Err(err) = writer.flush().await {
      event.send_modify(RenderEvent::as_handled);
      logs.add(Log::error(err)).await;
    }

    if total_saved >= 1000 {
      logs.add(Log::info(format!("Saved {total_saved} records"))).await;
      total_saved = 0;
    }
  }

  never!()
}
