use std::path::PathBuf;

use helper::UnhandledEvent;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::watch::Sender;
use tokio::{fs, select};

use crate::app::handler::BucketStatus;
use crate::app::{MpscRx, WatchRx};
use crate::never;

pub async fn output_writer(mut bucket_rx: MpscRx<BucketStatus>, mut output_rx: WatchRx<PathBuf>, event: Sender<UnhandledEvent>) {
  let mut output = output_rx.borrow_and_update().clone();
  loop {
    select! {
      new_output = output_rx.wait_for(|path| *path != output) => output = new_output.unwrap().clone(),
      _ = writer(&mut bucket_rx, &output, &event) => continue
    }
  }
}

pub async fn writer(rx: &mut MpscRx<BucketStatus>, output: &PathBuf, event: &Sender<UnhandledEvent>) {
  let file = match fs::File::options().create(true).append(true).write(true).open(output).await {
    Ok(file) => file,
    Err(err) => {
      event.send_modify(|e| *e = UnhandledEvent::from(err));
      never!()
    }
  };

  let mut writer = BufWriter::new(file);
  while let Some(bucket) = rx.recv().await.map(String::from) {
    if let Err(err) = writer.write_all(bucket.as_bytes()).await {
      event.send_modify(|e| *e = UnhandledEvent::from(err))
    }

    if let Err(err) = writer.write_all(b"\n").await {
      event.send_modify(|e| *e = UnhandledEvent::from(err))
    }
  }

  never!()
}
