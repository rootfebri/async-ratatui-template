use std::path::Path;

use helper::RenderEvent;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{fs, select};

use crate::app::app::impls::change_listener;
use crate::app::handler::sectrails::jsons::Record;
use crate::app::{MpscRx, WatchTx};
use crate::widgets::{Log, Logs};
use crate::{ARGS, never, wait_process};

pub async fn output_writer(mut bucket_rx: MpscRx<Record>, event: WatchTx<RenderEvent>, logs: Logs) {
  let mut output_path = ARGS.read().await.output.clone();

  loop {
    select! {
      _ = change_listener(|args| args.input.as_deref() == output_path.as_deref()) => output_path = ARGS.read().await.input.clone(),
      _ = writer(&mut bucket_rx, output_path.as_deref(), &event, logs.clone()) => continue
    }
  }
}

pub async fn writer(rx: &mut MpscRx<Record>, output: Option<impl AsRef<Path>>, event: &WatchTx<RenderEvent>, logs: Logs) {
  let Some(output) = output else { never!() };
  wait_process!();
  let info = format!("Writer working on `{}`", output.as_ref().display());
  logs.add(Log::info(info)).await;

  let file = match fs::File::options().create(true).append(true).open(&output).await {
    Ok(file) => {
      logs.add(Log::info(format!("Opened file: {}", output.as_ref().display()))).await;
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
      event.send_modify(RenderEvent::make_handled);

      total_saved += 1;
    }

    if let Err(err) = writer.write_all(b"\n").await {
      event.send_modify(RenderEvent::make_handled);
      logs.add(Log::error(err)).await;
    }
    if let Err(err) = writer.flush().await {
      event.send_modify(RenderEvent::make_handled);
      logs.add(Log::error(err)).await;
    }

    if total_saved >= 1000 {
      logs.add(Log::info(format!("Saved {total_saved} records"))).await;
      total_saved = 0;
    }
  }

  never!()
}
