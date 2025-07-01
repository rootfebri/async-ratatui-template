use std::sync::atomic::{AtomicU8, Ordering};

use crate::widgets::pulse::PulseLevel;
use strum::{Display, EnumIs};

#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Display, EnumIs)]
pub enum State {
  #[default]
  Iddling,
  Processing,
  Exit,
}

pub static SYNC_STATE: SyncState = SyncState(AtomicU8::new(SyncState::IDLE));

#[derive(Default)]
pub struct SyncState(AtomicU8);
impl SyncState {
  const EXIT: u8 = 0b000001;
  const IDLE: u8 = 0b000000;
  const PROCESS: u8 = 0b000010;

  pub fn as_pulse_level(&self) -> PulseLevel {
    match self.0.load(Ordering::SeqCst) {
      Self::EXIT => PulseLevel::Red,
      Self::PROCESS => PulseLevel::Green,
      _ => PulseLevel::Black,
    }
  }

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
}

impl AsRef<str> for SyncState {
  fn as_ref(&self) -> &str {
    match self.0.load(Ordering::SeqCst) {
      Self::EXIT => "EXITING",
      Self::IDLE => "IDLING",
      Self::PROCESS => "PROCESSING",
      _ => "",
    }
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
