use ratatui::style::{Color, Stylize};
use ratatui::text::Span;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse {
  pub page_props: PageProps,
}

impl PageResponse {
  pub fn into_records(self) -> Vec<Record> {
    self.page_props.server_response.data.records
  }
  pub fn as_records(&self) -> &[Record] {
    &self.page_props.server_response.data.records
  }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
  pub server_response: ServerResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerResponse {
  pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
  pub records: Vec<Record>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
  pub host_provider: Vec<String>,
  pub hostname: String,
  pub mail_provider: Vec<String>,
  pub open_page_rank: Option<isize>,
}

impl Record {
  pub fn as_spans(&self) -> [Span; 10] {
    [
      Span::raw("🌐"),
      Span::raw(" "),
      Span::raw(self.hostname.as_str()).fg(Color::Blue),
      Span::raw(" "),
      Span::raw("("),
      Span::raw(self.host_provider.first().map(String::as_str).unwrap_or("❔"))
        .fg(Color::Rgb(0, 0, 0))
        .italic(),
      Span::raw(")"),
      Span::raw(" "),
      Span::raw("📈"),
      Span::raw(self.mail_provider.first().map(String::as_str).unwrap_or("❔")),
    ]
  }

  pub fn as_csv(&self) -> String {
    format!(
      "{host_provider},{},{mail_provider},{rank}",
      self.hostname,
      host_provider = self.host_provider.join("|"),
      mail_provider = self.mail_provider.join("|"),
      rank = self.open_page_rank.unwrap_or_default()
    )
  }
}
