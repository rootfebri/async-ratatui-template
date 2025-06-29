use std::sync::Arc;

use anyhow::Error;
use ratatui::prelude::{Color, Stylize};
use ratatui::text::{Line, Span, ToSpan};
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

  fn timestamp_span(&self) -> [Span; 3] {
    match *self {
      Log::Bucket(ref bucket) => bucket.timestamp().as_spans(),
      Log::Info { ref timestamp, .. } | Log::Error { ref timestamp, .. } | Log::Warn { ref timestamp, .. } => timestamp.as_spans(),
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
    let mut spans = log.timestamp_span().to_vec();

    let content = match log {
      Log::Bucket(bucket) => return ListItem::from(bucket),
      Log::Info { line, .. } => Span::from(line.as_ref()).fg(log.color_content()),
      Log::Warn { line, .. } => Span::from(line.as_ref()).fg(log.color_content()),
      Log::Error { error, .. } => error.to_span().fg(log.color_content()),
    };

    spans.push(content);

    ListItem::from(Line::from_iter(spans))
  }
}
