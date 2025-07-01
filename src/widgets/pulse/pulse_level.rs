use super::*;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PulseLevel {
  Red,
  Green,
  #[default]
  Black,
}

impl PulseLevel {
  #[inline]
  pub const fn as_colors(&self) -> [Color; 10] {
    match *self {
      PulseLevel::Red => REDS,
      PulseLevel::Green => GREENS,
      PulseLevel::Black => BLACKS,
    }
  }
}

pub(super) const REDS: [Color; 10] = [
  Color::from_u32(0x00FF0000),
  Color::from_u32(0x00FF3333),
  Color::from_u32(0x00FF6666),
  Color::from_u32(0x00FF9999),
  Color::from_u32(0x00FFCCCC),
  Color::from_u32(0x00FF9999),
  Color::from_u32(0x00FF6666),
  Color::from_u32(0x00FF3333),
  Color::from_u32(0x00FF0000),
  Color::from_u32(0x00990000),
];
pub(super) const BLACKS: [Color; 10] = [
  Color::from_u32(0x00888888),
  Color::from_u32(0x009A9A9A),
  Color::from_u32(0x00ACACAC),
  Color::from_u32(0x00BEBEBE),
  Color::from_u32(0x00D0D0D0),
  Color::from_u32(0x00BEBEBE),
  Color::from_u32(0x00ACACAC),
  Color::from_u32(0x009A9A9A),
  Color::from_u32(0x00888888),
  Color::from_u32(0x006E6E6E),
];
pub(super) const GREENS: [Color; 10] = [
  Color::from_u32(0x0000FF00),
  Color::from_u32(0x0033FF33),
  Color::from_u32(0x0066FF66),
  Color::from_u32(0x0099FF99),
  Color::from_u32(0x00CCFFCC),
  Color::from_u32(0x0099FF99),
  Color::from_u32(0x0066FF66),
  Color::from_u32(0x0033FF33),
  Color::from_u32(0x0000FF00),
  Color::from_u32(0x00009900),
];
