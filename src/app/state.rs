use strum::{Display, EnumIs};

#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Display, EnumIs)]
pub enum State {
  #[default]
  Iddling,
  Processing,
  Exit,
}
