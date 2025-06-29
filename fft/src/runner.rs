use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs::read_dir;
use tokio::select;
use tokio::sync::{RwLock, watch};
use tokio::task::JoinHandle;

use super::state::NewChannels;
use super::widget::ExplorerContent;

#[allow(dead_code)]
pub struct Runner {
  cwd_handle: JoinHandle<()>,
  list_handle: JoinHandle<()>,
}

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

impl Runner {
  pub fn new(new_channels: NewChannels, tree: Arc<RwLock<Vec<ExplorerContent>>>) -> Self {
    let cwd_handle = tokio::spawn(parent_content_scanner(new_channels.parrent_watch, tree));
    let list_handle = tokio::spawn(child_content_scanner(new_channels.child_watch));
    Self { cwd_handle, list_handle }
  }
}

async fn parent_content_scanner(mut cur_parent_rx: watch::Receiver<Option<PathBuf>>, tree: Arc<RwLock<Vec<ExplorerContent>>>) {
  // Initialize with the current value from the watch channel
  let mut parent = cur_parent_rx.borrow().clone();

  async fn scanner(path: &Option<PathBuf>, cwd_list: &RwLock<Vec<ExplorerContent>>) {
    if let Some(path) = path
      && path.is_dir()
    {
      cwd_list.write().await.clear();
      let mut readdir = value_or_never!(read_dir(path).await);

      while let Some(entry) = value_or_never!(readdir.next_entry().await) {
        let child = if entry.path().is_file() {
          ExplorerContent::File {
            content: Default::default(),
            path: Arc::from(entry.path()),
            state: Default::default(),
          }
        } else {
          ExplorerContent::Dir {
            path: Arc::from(entry.path()),
          }
        };

        cwd_list.write().await.push(child);
      }

      never!()
    }
  }

  async fn watcher(watcher: &mut watch::Receiver<Option<PathBuf>>, current: &Option<PathBuf>) -> bool {
    let current_canon = current.as_deref().map(Path::canonicalize).and_then(Result::ok);
    watcher
      .wait_for(|cur| cur.as_deref().map(Path::canonicalize).and_then(Result::ok) != current_canon)
      .await
      .is_ok()
  }

  loop {
    select! {
      _ = scanner(&parent, &tree) => {},
      true = watcher(&mut cur_parent_rx, &parent) => parent = cur_parent_rx.borrow().clone(),
      else => break
    }
  }
}

async fn child_content_scanner(mut cur_child_tx: watch::Receiver<Option<ExplorerContent>>) {
  let mut ch = cur_child_tx.borrow_and_update().clone();

  async fn scanner(ch: Option<&ExplorerContent>) {
    if let Some(exc) = ch {
      exc.auto_load().await;
    }
  }

  async fn watcher(watcher: &mut watch::Receiver<Option<ExplorerContent>>, ch: &Option<ExplorerContent>) {
    _ = watcher.wait_for(|cur| cur != ch).await;
  }

  loop {
    select! {
      _ = scanner(ch.as_ref()) => tokio::time::sleep(tokio::time::Duration::from_millis(10)).await,
      _ = watcher(&mut cur_child_tx, &ch) => ch = cur_child_tx.borrow().clone(),
    }
  }
}
