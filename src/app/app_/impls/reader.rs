use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use helper::UnhandledEvent;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::watch::Sender;
use tokio::time::sleep;
use tokio::{fs, select};

use crate::app::{MpscTx, State, WatchRx};
use crate::never;
use crate::widgets::{Log, Logs, Statistic};

pub async fn input_reader(
  line_tx: MpscTx<Arc<str>>,
  mut watched_input: WatchRx<PathBuf>,
  event: Sender<UnhandledEvent>,
  logs: Logs,
  statistic: Statistic,
  state_watcher: WatchRx<State>,
) {
  let info = format!("Input reader spawn with `{}` value", watched_input.borrow_and_update().display());
  logs.add(Log::info(info)).await;

  let mut input = watched_input.borrow_and_update().clone();

  loop {
    select! {
      new_input = watched_input.wait_for(|current| *current != input) => input = new_input.unwrap().clone(),
      _ = read(&input, &line_tx, &event, logs.clone(), statistic.clone(), &state_watcher) => {}
    }
  }
}

pub async fn read(
  path: impl AsRef<Path>,
  sender: &MpscTx<Arc<str>>,
  event: &Sender<UnhandledEvent>,
  logs: Logs,
  statistic: Statistic,
  state_watcher: &WatchRx<State>,
) -> Result<()> {
  let info = format!("Input reader started reading `{}`", path.as_ref().display());
  logs.add(Log::info(info)).await;

  let mut file = match fs::File::open(&path).await {
    Ok(file) => file,
    Err(err) => {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = UnhandledEvent::render());
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
        event.send_modify(|e| *e = UnhandledEvent::render());
        never!()
      }
    }
    Err(err) => {
      logs.add(Log::error(err)).await;
      event.send_modify(|e| *e = UnhandledEvent::render());
      never!()
    }
  }

  let mut lines = BufReader::new(file).lines();

  loop {
    let true = state_watcher.borrow().is_processing() else {
      sleep(Duration::from_millis(16)).await;
      continue;
    };

    match lines.next_line().await {
      Ok(None) => break,
      Ok(Some(next_line)) => {
        let line = next_line.trim();
        if line.is_empty() {
          continue;
        } else if let Err(err) = sender.send(line.into()).await {
          event.send_modify(|e| *e = UnhandledEvent::error(err.to_string().into()));
        }
      }
      Err(err) => event.send_modify(|e| *e = UnhandledEvent::from(err)),
    }
  }

  let info = format!("Input reader finished reading `{}`", path.as_ref().display());
  logs.add(Log::info(info)).await;

  never!()
}
