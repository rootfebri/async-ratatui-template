#[derive(Debug, Copy, Default, Clone, Eq, PartialEq, Hash)]
pub enum PulseFps {
  #[default]
  /// 15
  VeryLow = 15,
  /// 30
  Low = 30,
  /// 45
  Normal = 45,
  /// 60
  High = 60,
  /// 90
  VeryHigh = 90,
}

impl From<u8> for PulseFps {
  fn from(value: u8) -> Self {
    match value {
      0..=15 => Self::Low,
      16..=30 => Self::Low,
      31..=45 => Self::Low,
      46..=60 => Self::Low,
      61..=90 => Self::Low,
      _ => Self::Low,
    }
  }
}
