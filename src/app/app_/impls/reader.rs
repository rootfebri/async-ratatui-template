use helper::RenderEvent;
use std::io::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::watch::Sender;
use tokio::{fs, select};

use crate::app::MpscTx;
use crate::app::app_::impls::change_listener;
use crate::widgets::{Log, Logs, Statistic};
use crate::{ARGS, never, wait_process};

pub async fn input_reader(line_tx: MpscTx<Arc<str>>, event: Sender<RenderEvent>, logs: Logs, statistic: Statistic) {
  let mut input_path = ARGS.read().await.input.clone();

  loop {
    select! {
      _ = change_listener(|args| args.input.as_deref() == input_path.as_deref()) => input_path = ARGS.read().await.input.clone(),
      _ = read(input_path.as_deref(), &line_tx, &event, logs.clone(), statistic.clone()) => {}
    }
  }
}

pub async fn read(
  path: Option<impl AsRef<Path>>,
  sender: &MpscTx<Arc<str>>,
  event: &Sender<RenderEvent>,
  logs: Logs,
  statistic: Statistic,
) -> Result<()> {
  let Some(ref_path) = path else { never!() };

  wait_process!();
  let info = format!("Input reader started reading `{}`", ref_path.as_ref().display());
  logs.add(Log::info(info)).await;

  let mut file = match fs::File::open(&ref_path).await {
    Ok(file) => file,
    Err(err) => {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = RenderEvent::render());
      never!();
    }
  };

  let mut content = String::new();
  match file.read_to_string(&mut content).await {
    Ok(_) => {
      statistic.set_max(content.chars().filter(|chr| chr == &'\n').count());
      statistic.set_current(0);
      if let Err(err) = file.rewind().await {
        logs.add(Log::error(err)).await;
        event.send_modify(|e| *e = RenderEvent::render());
        never!()
      }
    }
    Err(err) => {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = RenderEvent::render());
      never!()
    }
  }

  let mut lines = BufReader::new(file).lines();

  loop {
    wait_process!();

    match lines.next_line().await {
      Ok(None) => break,
      Ok(Some(next_line)) => {
        let line = next_line.trim();
        if line.is_empty() {
          continue;
        } else if let Err(err) = sender.send(line.into()).await {
          logs.add(Log::error(err)).await;
        }
      }
      Err(err) => event.send_modify(|e| *e = RenderEvent::from(err)),
    }
    event.send_modify(|e| *e = RenderEvent::render());
  }

  let info = format!("Input reader finished reading `{}`", ref_path.as_ref().display());
  logs.add(Log::info(info)).await;

  never!()
}
