use std::fmt::{Display, Formatter};
use std::sync::Arc;

use Status::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Widget};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;
use strum::{Display, EnumIs};

use super::*;
use crate::widgets::Timestamp;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct BucketStatus {
  name: Arc<str>,
  region: Region,
  status: Status,
  code: StatusCode,
  check_date: Timestamp,
}

impl BucketStatus {
  const BASE: &'static str = "https://s3.region.amazonaws.com/bucket";
  pub async fn new(name: impl Into<Arc<str>>) -> Result<Self> {
    let name = name.into();
    let check_date = Timestamp::now();
    let mut region = Region::UsEast1;
    let url = Self::BASE.replace("region", region.as_ref()).replace("bucket", name.as_ref());

    let response = head(&url).await?;
    if response.status().as_u16() == 404 {
      return Ok(BucketStatus {
        name: name.clone(),
        region: Region::from_ip(name.as_ref()).await.unwrap_or(region),
        status: Available,
        code: StatusCode::NOT_FOUND,
        check_date,
      });
    }

    if response.status().is_redirection() {
      match response
        .headers()
        .get("x-amz-bucket-region")
        .and_then(|h| Some(Region::from(h.to_str().ok()?)))
      {
        Some(r) => {
          region = r;
        }
        None => {
          return Ok(Self {
            name,
            region,
            status: Unknown,
            code: response.status(),
            check_date,
          });
        }
      }
    }

    let url = Self::BASE.replace("region", region.as_ref()).replace("bucket", &name);
    let response = head(&url).await?;
    let status: Status = response.status().into();
    let region = if !status.is_available() { None } else { Region::from_ip(&name).await }.unwrap_or(region);

    Ok(Self {
      name,
      region,
      status,
      code: response.status(),
      check_date,
    })
  }
  pub fn timestamp(&self) -> Timestamp {
    self.check_date
  }
}

impl Display for BucketStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    use ratatui::symbols::line::DOUBLE_VERTICAL_LEFT as SEP;

    write!(f, "{}{}{}", self.status, SEP, self.name)
  }
}

impl From<BucketStatus> for String {
  fn from(value: BucketStatus) -> Self {
    value.to_string()
  }
}

impl<'s> From<&'s BucketStatus> for Line<'s> {
  fn from(value: &'s BucketStatus) -> Self {
    let mut spans: Vec<Span> = vec![
      Span::raw(ratatui::symbols::line::DOUBLE_VERTICAL_LEFT).fg(Color::DarkGray),
      value.status.into(),
      Span::raw(ratatui::symbols::line::DOUBLE_VERTICAL_RIGHT).fg(Color::DarkGray),
      Span::raw(value.name.as_ref()),
      Span::raw("("),
      Span::raw(value.region.to_string()),
      Span::raw(")"),
    ];

    for (pos, span) in value.check_date.as_spans().into_iter().enumerate() {
      spans.insert(pos, span);
    }

    Line::from_iter(spans)
  }
}

impl<'s> From<&'s BucketStatus> for ListItem<'s> {
  fn from(value: &'s BucketStatus) -> Self {
    let line: Line<'s> = value.into();
    Self::new(line)
  }
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Display, EnumIs)]
enum Status {
  #[default]
  Unknown,
  Available,
  Unavailable,
}
impl From<StatusCode> for Status {
  fn from(status: StatusCode) -> Self {
    if status.is_success() {
      Unavailable
    } else if status.is_redirection() {
      Unknown
    } else {
      match status.as_u16() {
        404 => Available,
        403 => Unavailable,
        _ => Unknown,
      }
    }
  }
}
impl From<Status> for Span<'static> {
  fn from(value: Status) -> Self {
    match value {
      Unknown => Span::raw("❔"),
      Available => Span::raw("✅"),
      Unavailable => Span::raw("⛔"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_available_bucket() {
    let domain = "zvonar.dev";
    let bucket_status = BucketStatus::new(domain).await.unwrap();
    assert!(bucket_status.status.is_unavailable());
    assert_eq!(bucket_status.code.as_u16(), 403);
  }
}
