use std::net::Ipv4Addr;
use std::sync::LazyLock;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::*;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

mod bucket_status;
pub use bucket_status::*;

mod aws_ranges;
pub use aws_ranges::*;

pub static TAR: LazyLock<TokioAsyncResolver> = LazyLock::new(|| TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default()));

macro_rules! ttlen {
  ($tt:tt) => {
    1usize
  };
}

macro_rules! header_static {
  {
    $name:ident = [
      $($key:literal: $value:literal $(,)?)*
    ]
  } => {
    const $name: [(HeaderName, HeaderValue); $(ttlen!($key)+)+0usize] = [
      $((::reqwest::header::HeaderName::from_static($key), ::reqwest::header::HeaderValue::from_static($key))),+
    ];
  };
}

header_static! {
  BUCKET_HEAD = [
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:139.0) Gecko/20100101 Firefox/139.0",
    "accept": "*/*",
    "accept-language": "en-US,en;q=0.5",
    "x-amz-user-agent": "aws-sdk-js/2.1687.0",
    "x-amz-s3-console-op-name": "HeadBucket",
    "s3v3": "s3v3",
    "origin": "https://us-east-1.console.aws.amazon.com",
    "sec-gpc": "1",
    "connection": "keep-alive",
    "referer": "https://us-east-1.console.aws.amazon.com/",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "cross-site",
  ]
}

#[inline(always)]
async fn head(u: impl IntoUrl) -> Result<Response> {
  Client::builder()
    .default_headers(HeaderMap::from_iter(BUCKET_HEAD))
    .build()?
    .head(u)
    .send()
    .await
}

pub async fn get_ip(domain: addr::domain::Name<'_>) -> Option<Ipv4Addr> {
  let lookup = TAR.ipv4_lookup(domain.as_str()).await.ok()?;
  let a = lookup.as_lookup().records().iter().find(|r| r.data().is_some_and(|d| d.is_a()))?;
  Some(a.data()?.as_a()?.0)
}
