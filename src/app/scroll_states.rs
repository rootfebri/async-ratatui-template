pub type VerticalScroll = u16;
pub type HorizontalScroll = u16;
pub type ParagraphScroll = (VerticalScroll, HorizontalScroll);

#[derive(Debug, Default, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScrollStates {
  pub(crate) input_widget: ParagraphScroll,
  pub(crate) output_widget: ParagraphScroll,
  pub(crate) activity: ParagraphScroll,
}
