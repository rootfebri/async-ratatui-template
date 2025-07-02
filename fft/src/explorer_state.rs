use super::*;
use crossterm::event::KeyEvent;
use helper::{RenderEvent, keys};
use ratatui::widgets::ListState;
use std::ops::Deref;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, watch};
use tokio::task::JoinHandle;
use tokio::time::sleep;

impl Default for ExplorerState {
  fn default() -> Self {
    Self::new(None)
  }
}

pub struct ExplorerState {
  #[allow(dead_code)]
  dir_scanner: JoinHandle<()>,
  pub(crate) watch_dir: watch::Sender<Arc<Path>>,

  pub(crate) cursor: usize, // byte position in input string
  pub(crate) input_state: InputState,

  pub(crate) list: Arc<RwLock<Vec<ExplorerContent>>>,

  pub(crate) list_state: ListState,
}

impl ExplorerState {
  pub fn new(entry: impl Into<Option<PathBuf>>) -> Self {
    let canonicalized = Self::realpath_blocking(entry.into().unwrap_or_else(|| PathBuf::from("./")));
    let (watch_tx, watch_rx) = watch::channel(Arc::from(canonicalized));
    let list: Arc<RwLock<Vec<ExplorerContent>>> = Default::default();

    Self {
      dir_scanner: tokio::spawn(parent_content_scanner(watch_rx, list.clone())),
      watch_dir: watch_tx,
      cursor: 0,
      input_state: InputState::new(""),
      list,
      list_state: ListState::default(),
    }
  }

  async fn update_watched_cwd(&mut self, path: impl Into<PathBuf>) -> Option<RenderEvent> {
    let path = path.into();
    if !path.is_dir() {
      return None;
    }

    let canonicalized = Self::realpath(path).await;

    self.list.write().await.clear();
    self.watch_dir.send_modify(|dir| *dir = Arc::from(canonicalized));

    // Give the scanner a moment to start processing
    sleep(Duration::from_millis(16)).await;
    // Sync the new list contents and ensure selection
    self.ensure_selection().await;

    Some(RenderEvent::render())
  }

  async fn selected_content(&self) -> Option<ExplorerContent> {
    self.read_items().await.get(self.list_state.selected()?).cloned()
  }

  pub fn handle_paste(&mut self, content: &str) -> RenderEvent {
    self.input_state.push_str(content);
    RenderEvent::render()
  }

  pub async fn handle_key(&mut self, event: KeyEvent) -> Option<RenderEvent> {
    self.ensure_selection().await;

    match event {
      keys!(Up, NONE, Press) => self.list_state.select_next(),
      keys!(Down, NONE, Press) => self.list_state.select_previous(),
      keys!(Left, NONE, Press) => self.input_state.left(),
      keys!(Right, NONE, Press) => self.input_state.right(),

      keys!(Backspace, NONE, Press) => {
        self.input_state.backspace();
        self.clamp_selection_async().await;
      }
      keys!(Delete, NONE, Press) => {
        self.input_state.delete();
        self.clamp_selection_async().await;
      }
      keys!(Backspace, CONTROL, Press) => {
        self.input_state.ctrl_backspace();
        self.clamp_selection_async().await;
      }

      keys!(Delete, CONTROL, Press) => {
        self.input_state.ctrl_delete();
        self.clamp_selection_async().await;
      }
      keys!(Home, NONE, Press) => self.input_state.set_cursor(0),
      keys!(End, NONE, Press) => self.input_state.set_cursor(self.input_state.len()),
      keys!(Left, CONTROL, Press) => self.input_state.move_left_word(),
      keys!(Right, CONTROL, Press) => self.input_state.move_right_word(),
      keys!(Esc, NONE, Press) => return Some(RenderEvent::canceled()),

      keys!(Enter, NONE, Press) => return Some(RenderEvent::handled()),
      keys!(Left, ALT, Press) => {
        let mut cwd = self.watch_dir.borrow().to_path_buf();
        return if cwd.pop() { self.update_watched_cwd(cwd).await } else { None };
      }
      keys!(Right, ALT, Press) => {
        let content = self.selected_content().await.and_then(|c| if c.is_dir() { Some(c) } else { None })?;
        return self.update_watched_cwd(content.as_path().to_path_buf()).await;
      }

      keys!(Char('u'), CONTROL, Press) => {
        self.input_state.clear();
        self.clamp_selection_async().await;
      }
      keys!(Char(chr), NONE, Press) => {
        self.input_state.push(chr);
        self.clamp_selection_async().await;
      }

      _ => return None,
    }

    // Always normalize cursor after text operations
    self.input_state.normalize_cursor();
    Some(RenderEvent::render())
  }

  pub async fn get(self) -> Option<PathBuf> {
    self
      .read_items()
      .await
      .get(self.list_state.selected()?)
      .map(|content| content.as_path().to_path_buf())
  }

  pub(crate) fn selected_content_blocking(&self) -> Option<ExplorerContent> {
    self.blocking_read_items().get(self.list_state.selected()?).cloned()
  }

  pub(crate) fn blocking_read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.blocking_read();
    let mut items = list
      .iter()
      .filter(|item| self.input_state.is_empty() || item.filename().fuzzy_contains(self.input_state.deref()))
      .map(ExplorerContent::clone)
      .collect::<Vec<_>>();
    drop(list);

    items.sort_unstable_by(|a, b| match (a.is_file(), b.is_file()) {
      (true, true) => Equal,
      (true, false) | (false, true) => Greater,
      (false, false) => Less,
    });

    if !self.input_state.is_empty() {
      items.sort_unstable_by(|a, b| {
        let a_score = a.filename().fuzzy_score(self.input_state.deref());
        let b_score = b.filename().fuzzy_score(self.input_state.deref());
        a_score.cmp(&b_score)
      });
    }

    items
  }

  pub(crate) async fn read_items(&self) -> Vec<ExplorerContent> {
    use std::cmp::Ordering::*;

    let list = self.list.read().await;
    let mut items = list
      .iter()
      .filter(|item| self.input_state.is_empty() || item.filename().fuzzy_contains(self.input_state.deref()))
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

  /// Ensure we have a valid selection when items are available (async version)
  async fn ensure_selection(&mut self) {
    self.clamp_selection_async().await;
  }

  /// Ensure the selection is within bounds of the filtered list (async version)
  async fn clamp_selection_async(&mut self) {
    let filtered_items = self.read_items().await;
    if filtered_items.is_empty() {
      self.list_state.select(None);
    } else if let Some(selected) = self.list_state.selected() {
      if selected >= filtered_items.len() {
        self.list_state.select(Some(filtered_items.len() - 1));
      }
    } else {
      // No selection but items exist, select first one
      self.list_state.select(Some(0));
    }
  }

  async fn realpath(path: PathBuf) -> PathBuf {
    if !path.is_relative() {
      return path;
    }

    let Ok(canonicalized) = tokio::fs::canonicalize(&path).await else {
      return path;
    };
    canonicalized
      .components()
      .filter(|c| !matches!(c, std::path::Component::Prefix(_)))
      .collect()
  }

  fn realpath_blocking(path: PathBuf) -> PathBuf {
    if !path.is_relative() {
      return path;
    }

    let Ok(canonicalized) = path.canonicalize() else { return path };
    canonicalized
      .components()
      .filter(|c| !matches!(c, std::path::Component::Prefix(_)))
      .collect()
  }
}
