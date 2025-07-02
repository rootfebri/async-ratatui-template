use std::cell::RefCell;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScrollState {
  pub auto_scroll: bool,
  pub horizontal: u16,
  pub vertical: u16,
}

impl ScrollState {
  pub fn as_tuple(&self) -> (u16, u16) {
    (self.vertical, self.horizontal)
  }

  pub fn auto_scroll(&mut self, max_items: usize, visible_height: usize) {
    if self.auto_scroll && max_items > visible_height {
      self.vertical = (max_items - visible_height) as u16;
    }
  }

  pub fn scroll_to_top(&mut self) {
    self.vertical = 0;
    self.lock();
  }

  pub fn unlock(&mut self) {
    self.auto_scroll = true;
  }
  pub fn lock(&mut self) {
    self.auto_scroll = false;
  }

  pub fn scroll_up_by(&mut self, len: u16, _max_scroll: u16) {
    self.vertical = self.vertical.saturating_sub(len);
    self.lock();
  }

  pub fn scroll_down_by(&mut self, len: u16, max_scroll: u16) {
    self.vertical = self.vertical.saturating_add(len).min(max_scroll);
    self.lock();
  }

  pub fn scroll_left_by(&mut self, len: u16) {
    self.horizontal = self.horizontal.saturating_sub(len);
  }
  pub fn scroll_right_by(&mut self, len: u16) {
    self.horizontal = self.horizontal.saturating_add(len);
  }

  pub fn scroll_up(&mut self, _max_scroll: u16) {
    self.vertical = self.vertical.saturating_sub(1);
    self.lock();
  }

  pub fn scroll_down(&mut self, max_scroll: u16) {
    self.vertical = self.vertical.saturating_add(1).min(max_scroll);
    self.lock();
  }

  pub fn scroll_left(&mut self) {
    self.horizontal = self.horizontal.saturating_sub(1);
    self.lock();
  }

  pub fn scroll_right(&mut self) {
    self.horizontal = self.horizontal.saturating_add(1);
    self.lock();
  }

  pub fn scroll_to_bottom(&mut self, max_scroll: u16) {
    self.vertical = max_scroll;
    self.unlock();
  }

  pub fn max_scroll_position(&self, items_count: usize, visible_height: usize) -> u16 {
    if items_count <= visible_height {
      0
    } else {
      (items_count - visible_height) as u16
    }
  }
}

impl Default for ScrollState {
  fn default() -> Self {
    Self {
      auto_scroll: true,
      horizontal: 0,
      vertical: 0,
    }
  }
}

#[derive(Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScrollStates {
  pub(crate) activity: RefCell<ScrollState>,
}
