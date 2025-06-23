use std::sync::Arc;

use anyhow::Error;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize, Text, Widget};
use ratatui::text::{Line, Span, ToLine, ToSpan};
use ratatui::widgets::ListItem;

use crate::app::handler::BucketStatus;
use crate::widgets::Timestamp;

#[derive(Debug)]
pub enum Log {
  Bucket(BucketStatus),
  Info { timestamp: Timestamp, line: Arc<str> },
  Error { timestamp: Timestamp, error: Error },
  Warn { timestamp: Timestamp, line: Arc<str> },
}

impl Log {
  pub fn as_list_item(&self) -> ListItem<'_> {
    self.into()
  }

  pub fn bucket(bucket: BucketStatus) -> Self {
    Self::Bucket(bucket)
  }
  pub fn warn(line: impl Into<Arc<str>>) -> Self {
    Self::Warn {
      timestamp: Default::default(),
      line: line.into(),
    }
  }
  pub fn info(line: impl Into<Arc<str>>) -> Self {
    Self::Info {
      timestamp: Default::default(),
      line: line.into(),
    }
  }
  pub fn error(error: impl Into<Error>) -> Self {
    Self::Error {
      timestamp: Default::default(),
      error: error.into(),
    }
  }

  fn timestamp_span(&self) -> Line<'_> {
    match *self {
      Log::Bucket(ref bucket) => bucket.timestamp().into(),
      Log::Info { ref timestamp, .. } | Log::Error { ref timestamp, .. } | Log::Warn { ref timestamp, .. } => timestamp.into(),
    }
  }

  fn color_content(&self) -> Color {
    match *self {
      Log::Bucket(_) => Color::Black,
      Log::Info { .. } => Color::Rgb(0, 251, 255),
      Log::Error { .. } => Color::Rgb(99, 0, 0),
      Log::Warn { .. } => Color::Rgb(99, 0, 0),
    }
  }
}

impl<'a> From<&'a Log> for ListItem<'a> {
  fn from(log: &'a Log) -> Self {
    let content = match log {
      Log::Bucket(bucket) => return ListItem::from(bucket).on_blue(),
      Log::Info { line, .. } => Line::from(line.as_ref()).fg(Color::Rgb(0, 251, 255)),
      Log::Warn { line, .. } => Line::from(line.as_ref()).fg(Color::Rgb(137, 82, 0)),
      Log::Error { error, .. } => Line::from(error.to_line()).fg(Color::Rgb(99, 0, 0)),
    };
    let prefix = log.timestamp_span();

    ListItem::from(prefix + content)
  }
}
