use super::*;
use crossterm::event::KeyEvent;
use helper::{RenderEvent, keys};
use ratatui::widgets::ListState;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tokio::sync::{RwLock, mpsc, watch};
use tokio::time::sleep;
use widget::ExplorerContent;

impl Default for ExplorerState {
  fn default() -> Self {
    let cwd = PathBuf::from("./");
    let (channels, new_channels) = Channels::new();
    channels.current_parrent_tx.send(Some(cwd.clone())).unwrap();

    let runners = Runner::new(new_channels);

    Self {
      runners,
      channels,
      cursor: 0,
      input: "".to_string(),
      cwd,
      list: Arc::new(Default::default()),
      list_state: ListState::default(),
    }
  }
}

pub struct ExplorerState {
  #[allow(dead_code)]
  pub(super) runners: Runner,
  pub(super) channels: Channels,

  pub(super) cursor: usize,
  pub(super) input: String,

  pub(super) cwd: PathBuf,
  pub(super) list: Arc<RwLock<Vec<ExplorerContent>>>,

  pub(super) list_state: ListState,
}

impl ExplorerState {
  async fn sync_list(&mut self) {
    if self.channels.mpsc_list.is_empty() {
      self
        .channels
        .mpsc_list
        .recv_many(self.list.write().await.as_mut(), self.channels.mpsc_list.len())
        .await;
    }
  }

  async fn update_watched_child(&mut self) {
    let selected_content = self.selected_content().await;
    self
      .channels
      .current_child_tx
      .send_modify(|current_child| *current_child = selected_content);
  }

  async fn update_watched_cwd(&mut self) {
    self.channels.current_child_tx.send_modify(|current_child| _ = current_child.take());
    self.channels.mpsc_list.recv_many(&mut vec![], self.channels.mpsc_list.len()).await;
    self.list.write().await.clear();
    self
      .channels
      .current_parrent_tx
      .send_modify(|current_cwd| *current_cwd = Some(self.cwd.clone()));

    sleep(Duration::from_millis(16)).await;

    self.sync_list().await;
    let selected_child = if let Some(i) = self.list_state.selected() {
      self.read_items().await.get(i).cloned()
    } else {
      return;
    };

    self
      .channels
      .current_child_tx
      .send_modify(|current_child| *current_child = selected_child);
  }

  async fn selected_content(&self) -> Option<ExplorerContent> {
    self.read_items().await.get(self.list_state.selected()?).cloned()
  }

  pub fn handle_paste(&mut self, content: &str) -> RenderEvent {
    for chr in content.chars() {
      self.input.push_char(chr, &mut self.cursor);
    }

    RenderEvent::render()
  }

  pub async fn handle_key(&mut self, event: KeyEvent) -> Option<RenderEvent> {
    self.sync_list().await;

    match event {
      keys!(Up, NONE, Press) => {
        self.list_state.select_previous();
        self.update_watched_child().await;
      }
      keys!(Down, NONE, Press) => {
        self.list_state.select_next();
        self.update_watched_child().await;
      }
      keys!(Left, NONE, Press) => self.cursor = self.cursor.saturating_sub(1),
      keys!(Right, NONE, Press) => self.cursor += self.cursor.add(1).min(self.input.chars().count()),

      keys!(Char(chr), NONE, Press) => self.input.push_char(chr, &mut self.cursor),
      keys!(Backspace, NONE, Press) if self.input.remove_char(self.cursor).is_some() => self.cursor = self.cursor.saturating_sub(1),
      keys!(Delete, NONE, Press) if self.input.remove_char(self.cursor + 1).is_some() => self.cursor = self.cursor.saturating_sub(1),
      keys!(Backspace, CONTROL, Press) => self.input.remove_word_backwards(&mut self.cursor),
      keys!(Delete, CONTROL, Press) => self.input.remove_word_forwards(self.cursor),

      keys!(Esc, NONE, Press) => return Some(RenderEvent::canceled()),
      keys!(Enter, NONE, Press) => return Some(RenderEvent::handled()),
      keys!(Left, ALT, Press) => {
        if self.cwd.pop() {
          self.update_watched_cwd().await;
        }

        return None;
      }
      keys!(Right, ALT, Press) => {
        if let Some(content) = self.selected_content().await
          && !content.is_file()
        {
          self.cwd = content.as_path().to_path_buf();
          self.update_watched_cwd().await;
        } else {
          return None;
        }
      }

      _ => return None,
    }

    Some(RenderEvent::render())
  }

  pub(super) fn selected_content_blocking(&self) -> Option<ExplorerContent> {
    self.blocking_read_items().get(self.list_state.selected()?).cloned()
  }

  pub(super) fn blocking_read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.blocking_read();
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.as_cow().fuzzy_contains(self.input.as_str()))
      .map(ExplorerContent::clone)
      .collect::<Vec<_>>();
    drop(list);

    items.sort_unstable_by(|a, b| match (a.is_file(), b.is_file()) {
      (true, true) => Equal,
      (true, false) | (false, true) => Greater,
      (false, false) => Less,
    });

    items
  }

  pub(super) async fn read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.read().await;
    let mut items = list
      .iter()
      .filter(|item| self.input.is_empty() || item.as_cow().fuzzy_contains(self.input.as_str()))
      .map(ExplorerContent::clone)
      .collect::<Vec<_>>();

    drop(list);

    items.sort_unstable_by(|a, b| match (a.is_file(), b.is_file()) {
      (true, true) => Equal,
      (true, false) | (false, true) => Greater,
      (false, false) => Less,
    });

    items
  }

  pub async fn get(self) -> Option<PathBuf> {
    self
      .read_items()
      .await
      .get(self.list_state.selected()?)
      .map(|content| content.as_path().to_path_buf())
  }
}

pub struct Channels {
  pub(super) current_parrent_tx: watch::Sender<Option<PathBuf>>,
  pub(super) current_child_tx: watch::Sender<Option<ExplorerContent>>,
  pub(super) mpsc_list: mpsc::Receiver<ExplorerContent>,
}
pub struct NewChannels {
  pub list: Sender<ExplorerContent>,
  pub parrent_watch: Receiver<Option<PathBuf>>,
  pub child_watch: Receiver<Option<ExplorerContent>>,
}

impl Channels {
  pub fn new() -> (Channels, NewChannels) {
    let (list_tx, mpsc_list) = mpsc::channel(1024);
    let (current_parrent_tx, current_parrent_rx) = watch::channel(Option::<PathBuf>::None);
    let (current_child_tx, current_child_rx) = watch::channel(Option::<ExplorerContent>::None);
    (
      Self {
        current_parrent_tx,
        mpsc_list,
        current_child_tx,
      },
      NewChannels {
        list: list_tx,
        parrent_watch: current_parrent_rx,
        child_watch: current_child_rx,
      },
    )
  }
}
