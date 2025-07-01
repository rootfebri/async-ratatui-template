use std::path::Path;
use std::sync::Arc;

use super::ExplorerContent;
use tokio::fs::read_dir;
use tokio::select;
use tokio::sync::{RwLock, watch};

macro_rules! never {
  () => {
    loop {
      ::tokio::time::sleep(tokio::time::Duration::from_secs(30)).await
    }
  };
}

macro_rules! value_or_never {
  ($result:expr) => {
    match $result {
      Ok(result) => result,
      Err(_) => never!(),
    }
  };
}

pub(super) async fn parent_content_scanner(mut cur_parent_rx: watch::Receiver<Arc<Path>>, tree: Arc<RwLock<Vec<ExplorerContent>>>) {
  // Initialize with the current value from the watch channel
  let mut parent = cur_parent_rx.borrow().clone();

  async fn scanner(path: &Arc<Path>, cwd_list: &RwLock<Vec<ExplorerContent>>) {
    if path.is_dir() {
      cwd_list.write().await.clear();
      let mut readdir = value_or_never!(read_dir(path).await);

      while let Some(entry) = value_or_never!(readdir.next_entry().await) {
        let explorer_content = ExplorerContent::async_new(entry.path()).await;
        cwd_list.write().await.push(explorer_content);
      }

      never!()
    }
  }

  async fn watcher(watcher: &mut watch::Receiver<Arc<Path>>, current: &Arc<Path>) -> bool {
    let current_canon = current.canonicalize().ok();
    watcher.wait_for(|cur| cur.canonicalize().ok() != current_canon).await.is_ok()
  }

  loop {
    select! {
      _ = scanner(&parent, &tree) => {},
      true = watcher(&mut cur_parent_rx, &parent) => parent = cur_parent_rx.borrow().clone(),
      else => break
    }
  }
}
