use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::app::SYNC_STATE;
use crate::ui::blk;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize, Widget};
use ratatui::symbols::border::Set;
use ratatui::text::{Line, Span, ToLine};
use ratatui::widgets::LineGauge;

impl Default for Statistic {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct Statistic {
  cur: Arc<AtomicUsize>,
  max: Arc<AtomicUsize>,
}

type InnerBlockArea = Rect;
impl Statistic {
  pub fn new() -> Self {
    Self {
      cur: Arc::new(AtomicUsize::new(0)),
      max: Arc::new(AtomicUsize::new(0)),
    }
  }

  pub fn set_current(&self, value: usize) {
    self.cur.store(value, Ordering::Relaxed);
  }

  pub fn set_max(&self, value: usize) {
    self.max.store(value, Ordering::Relaxed);
  }

  pub fn increment(&self) {
    self.cur.fetch_add(1, Ordering::Relaxed);
  }

  pub fn get_cur(&self) -> usize {
    self.cur.load(Ordering::Relaxed)
  }

  pub fn get_max(&self) -> usize {
    self.max.load(Ordering::Relaxed)
  }

  pub fn get_percentage(&self) -> f64 {
    if self.get_max() == 0 {
      0.0
    } else {
      (self.get_cur() as f64 / self.get_max() as f64 * 100f64).clamp(0f64, 100f64)
    }
  }

  pub fn get_percentage_abs(&self) -> u8 {
    let current = self.get_cur() as f64;
    let max = self.get_max() as f64;
    if max == 0.0 { 0 } else { (current / max * 100.0).min(100.0).abs() as u8 }
  }

  pub fn is_complete(&self) -> bool {
    self.get_cur() >= self.get_max()
  }

  fn draw_bar_percentage(&self) -> LineGauge {
    let indicator_line = [Span::raw(ratatui::symbols::line::HORIZONTAL), Span::raw(" ")]
      .into_iter()
      .chain(SYNC_STATE.as_spans())
      .chain([Span::raw(" "), Span::raw(ratatui::symbols::line::CROSS)])
      .collect::<Line>();

    let block = blk()
      .fg(Color::Reset)
      .title_top(indicator_line.left_aligned())
      .title_top(self.gauge_stats_line().right_aligned());

    LineGauge::default()
      .ratio((self.get_percentage()) / 100f64)
      .line_set(ratatui::symbols::line::NORMAL)
      .block(block)
      .filled_style(self.gauge_color())
      .unfilled_style(Style::default().fg(Color::DarkGray))
  }

  fn gauge_color(&self) -> Color {
    let percent = self.get_percentage().max(99.0);
    if SYNC_STATE.is_processing() {
      Color::from_u32(BRIGHT_COLOR[percent as usize])
    } else {
      Color::from_u32(DARK_COLOR[percent as usize])
    }
  }

  fn draw_outer_block(&self, area: Rect, buf: &mut Buffer) -> InnerBlockArea {
    use ratatui::symbols::line::DOUBLE_CROSS;

    let title_top_line = " 📊 Stats ".to_line().left_aligned();
    let main_border_set = Set {
      top_left: DOUBLE_CROSS,
      top_right: DOUBLE_CROSS,
      bottom_left: DOUBLE_CROSS,
      bottom_right: DOUBLE_CROSS,
      vertical_left: "│",
      vertical_right: "│",
      horizontal_top: "─",
      horizontal_bottom: "─",
    };

    let main_block = blk().title_top(title_top_line).border_set(main_border_set).border_style(Style {
      fg: Some(Color::Rgb(255, 102, 0)),
      ..Default::default()
    });

    (&main_block).render(area, buf);
    main_block.inner(area)
  }

  fn gauge_stats_line(&self) -> Line {
    let current = self.get_cur();
    let max = self.get_max();

    let spans = vec![
      Span::raw(" "),
      Span::raw(current.to_string()).fg(Color::Yellow),
      Span::raw(" / "),
      Span::raw(max.to_string()).fg(Color::White),
      Span::raw(" "),
    ];

    Line::from(spans)
  }
}

impl Widget for &Statistic {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let inner_area = self.draw_outer_block(area, buf);

    let [gauge_area, _] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(inner_area);

    self.draw_bar_percentage().render(gauge_area, buf);
  }
}

const DARK_COLOR: [u32; 100] = [
  0x00008000, 0x00038000, 0x00078000, 0x000A7F00, 0x000D7F00, 0x00117F00, 0x00147F00, 0x00177E00, 0x001B7E00, 0x001E7E00, 0x00217E00, 0x00247D00,
  0x00287D00, 0x002B7D00, 0x002E7D00, 0x00327C00, 0x00357C00, 0x00387C00, 0x003B7C00, 0x003F7B00, 0x00427B00, 0x00457B00, 0x00497B00, 0x004C7A00,
  0x004F7A00, 0x00527A00, 0x00567A00, 0x00597A00, 0x005C7900, 0x005F7900, 0x00637900, 0x00667900, 0x00697800, 0x006C7800, 0x00707800, 0x00737800,
  0x00767700, 0x00797700, 0x007D7700, 0x00807700, 0x00837600, 0x00867600, 0x008A7600, 0x008D7600, 0x00907500, 0x00937500, 0x00977500, 0x009A7500,
  0x009D7400, 0x00A07400, 0x00A47300, 0x00A77300, 0x00AA7300, 0x00AD7200, 0x00B17200, 0x00B47100, 0x00B77100, 0x00BA7000, 0x00BE7000, 0x00C16F00,
  0x00C46F00, 0x00C76F00, 0x00CB6E00, 0x00CE6E00, 0x00D16D00, 0x00D46D00, 0x00D86C00, 0x00DB6C00, 0x00DE6C00, 0x00E16B00, 0x00E56B00, 0x00E86A00,
  0x00EB6A00, 0x00EE6900, 0x00F26800, 0x00F56800, 0x00F86700, 0x00FB6700, 0x00FF6600, 0x00FF6500, 0x00FF6500, 0x00FF6400, 0x00FF6300, 0x00FF6200,
  0x00FF6200, 0x00FF6100, 0x00FF6000, 0x00FF5F00, 0x00FF5F00, 0x00FF5E00, 0x00FF5D00, 0x00FF5D00, 0x00FF5C00, 0x00FF5B00, 0x00FF5A00, 0x00FF5A00,
  0x00FF5900, 0x00FF5800, 0x00FF5700, 0x00FF5600,
];
const BRIGHT_COLOR: [u32; 100] = [
  0x0000FF00, 0x0003FC00, 0x0007F900, 0x000AF600, 0x000DF300, 0x0011F000, 0x0014ED00, 0x0017EA00, 0x001BE700, 0x001EE400, 0x0021E100, 0x0024DE00,
  0x0028DB00, 0x002BD800, 0x002ED500, 0x0032D200, 0x0035CF00, 0x0038CC00, 0x003BC900, 0x003FC600, 0x0042C300, 0x0045C000, 0x0049BD00, 0x004CBA00,
  0x004FB700, 0x0052B400, 0x0056B100, 0x0059AE00, 0x005CAB00, 0x005FA800, 0x0063A500, 0x0066A200, 0x00699F00, 0x006C9C00, 0x00709900, 0x00739600,
  0x00769300, 0x00798F00, 0x007D8C00, 0x00808900, 0x00838600, 0x00868300, 0x008A8000, 0x008D7D00, 0x00907A00, 0x00937700, 0x00977400, 0x009A7100,
  0x009D6E00, 0x00A06B00, 0x00A46800, 0x00A76500, 0x00AA6200, 0x00AD5F00, 0x00B15C00, 0x00B45900, 0x00B75600, 0x00BA5300, 0x00BE5000, 0x00C14D00,
  0x00C44A00, 0x00C74700, 0x00CB4400, 0x00CE4100, 0x00D13E00, 0x00D43B00, 0x00D83800, 0x00DB3500, 0x00DE3200, 0x00E12F00, 0x00E52C00, 0x00E82900,
  0x00EB2600, 0x00EE2300, 0x00F22000, 0x00F51D00, 0x00F81A00, 0x00FB1700, 0x00FF1400, 0x00FF1100, 0x00FF0E00, 0x00FF0B00, 0x00FF0800, 0x00FF0500,
  0x00FF0200, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000,
  0x00FF0000, 0x00FF0000, 0x00FF0000, 0x00FF0000,
];
