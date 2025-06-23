use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Block, BorderType, Clear, Widget};

pub fn fix_center(area: Rect, w: u16, h: u16) -> Rect {
  let [_, x, _] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(w), Constraint::Fill(1)]).areas(area);
  Layout::vertical([Constraint::Fill(1), Constraint::Length(h), Constraint::Fill(1)]).split(x)[1]
}

pub fn clear(area: Rect, buffer: &mut Buffer) {
  Clear.render(area, buffer);
}

#[inline(always)]
pub fn blk<'a>() -> Block<'a> {
  Block::bordered().border_type(BorderType::Rounded).fg(Color::Rgb(0, 255, 251))
}
