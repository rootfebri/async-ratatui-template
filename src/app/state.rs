use std::sync::atomic::{AtomicU8, Ordering};

use ratatui::prelude::Stylize;
use ratatui::style::Color;
use ratatui::text::Span;
use strum::{Display, EnumIs};

#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Display, EnumIs)]
pub enum State {
  #[default]
  Iddling,
  Processing,
  Exit,
}

pub static SYNC_STATE: SyncState = SyncState(AtomicU8::new(SyncState::IDLE));

static CS: ColorState = ColorState { cur: AtomicU8::new(0) };

struct ColorState {
  cur: AtomicU8,
}

impl ColorState {
  pub fn next(&self) -> Color {
    match self.cur.compare_exchange(10, 1, Ordering::SeqCst, Ordering::SeqCst) {
      Ok(_) => Color::Rgb(17, 17, 17),
      Err(cur) => {
        self.cur.swap(cur + 1, Ordering::Relaxed);
        match cur {
          1 => Color::Rgb(34, 34, 34),
          2 => Color::Rgb(85, 85, 85),
          3 => Color::Rgb(102, 102, 102),
          4 => Color::Rgb(153, 153, 153),
          5 => Color::Rgb(170, 170, 170),
          6 => Color::Rgb(204, 204, 204),
          7 => Color::Rgb(221, 221, 221),
          _ => Color::Rgb(238, 238, 238),
        }
      }
    }
  }
}

pub struct SyncState(AtomicU8);
impl SyncState {
  const EXIT: u8 = 0b000001;
  const IDLE: u8 = 0b000000;
  const PROCESS: u8 = 0b000010;

  pub fn is_exiting(&self) -> bool {
    self.0.load(Ordering::SeqCst) == Self::EXIT
  }

  pub fn is_idling(&self) -> bool {
    self.0.load(Ordering::SeqCst) == Self::IDLE
  }

  pub fn is_processing(&self) -> bool {
    self.0.load(Ordering::SeqCst) == Self::PROCESS
  }

  pub fn idle(&self) {
    self.0.swap(Self::IDLE, Ordering::SeqCst);
  }
  pub fn exit(&self) {
    self.0.swap(Self::EXIT, Ordering::SeqCst);
  }
  pub fn process(&self) {
    self.0.swap(Self::PROCESS, Ordering::SeqCst);
  }

  pub fn color_str(&self) -> (Color, &'static str) {
    match self.0.load(Ordering::SeqCst) {
      Self::IDLE => (CS.next(), "IDLE"),
      Self::PROCESS => (Color::Rgb(34, 255, 0), "PROCESS"),
      Self::EXIT => (Color::Red, "EXIT"),
      _ => unreachable!(),
    }
  }

  pub fn as_spans(&self) -> [Span<'static>; 3] {
    let (color, text) = self.color_str();
    [
      Span::raw(ratatui::symbols::DOT).fg(color),
      Span::raw(" "),
      Span::raw(text).fg(Color::White),
    ]
  }
}

#[macro_export]
macro_rules! wait_process {
  () => {
    while !$crate::app::SYNC_STATE.is_processing() {
      ::tokio::time::sleep(::tokio::time::Duration::from_millis(16)).await
    }
  };
}
