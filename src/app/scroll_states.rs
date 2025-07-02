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

  pub fn auto_scroll(&mut self) {
    if self.auto_scroll {
      self.scroll_down();
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

  pub fn scroll_up_by(&mut self, len: u16) {
    self.vertical = self.vertical.saturating_sub(len);
  }
  pub fn scroll_down_by(&mut self, len: u16) {
    self.vertical = self.vertical.saturating_add(len);
  }

  pub fn scroll_left_by(&mut self, len: u16) {
    self.horizontal = self.horizontal.saturating_sub(len);
  }
  pub fn scroll_right_by(&mut self, len: u16) {
    self.horizontal = self.horizontal.saturating_add(len);
  }

  pub fn scroll_up(&mut self) {
    self.vertical = self.vertical.saturating_sub(1);
  }
  pub fn scroll_down(&mut self) {
    self.vertical = self.vertical.saturating_add(1);
  }

  pub fn scroll_left(&mut self) {
    self.horizontal = self.horizontal.saturating_sub(1);
  }
  pub fn scroll_right(&mut self) {
    self.horizontal = self.horizontal.saturating_add(1);
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
