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
