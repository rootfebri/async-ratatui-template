use super::*;
use crate::ARGS;
use crate::app::handler::sectrails::jsons::PageResponse;
use anyhow::{Context, Result, bail};
use reqwest::cookie::Jar;
use reqwest::{Client, Response, Url};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct SecTrailClient {
  page: usize,
  data: Arc<str>,
  client: Client,
  expired: bool,
  base_url: Url,
}
impl Default for SecTrailClient {
  fn default() -> Self {
    Self::new()
  }
}
impl SecTrailClient {
  const BASE_URL: &'static str = "https://securitytrails.com/_next/data/";
  pub fn new() -> Self {
    let base_url = Url::parse(Self::BASE_URL).unwrap();

    Self {
      page: 1,
      data: Arc::default(),
      client: Default::default(),
      expired: true,
      base_url,
    }
  }

  pub fn new_sequence(&mut self, data: impl Into<Arc<str>>) {
    self.data = data.into();
    self.page = 1;
  }

  pub async fn get(&mut self) -> Result<PageResponse> {
    if self.expired {
      self.new_session().await.context("New sessionizer has failed!")?;
    }

    let url = self.next_url();
    let response = match self.client.get(url).send().await.and_then(Response::error_for_status) {
      Ok(response) => response,
      Err(error) => {
        if error.is_status() && error.status().unwrap().is_client_error() {
          tokio::time::sleep(Duration::from_secs(30)).await;
          bail!("Soft Error: Rate Limited and slept for 30s",);
        } else if error.is_request() {
          self.expired = true;
        }

        bail!(
          "Hard error: {error} | Happens while making a request for {} on page {}",
          self.data,
          self.page()
        );
      }
    };

    let body = response.bytes().await?;
    let page_response: PageResponse = match serde_json::from_slice(&body) {
      Ok(ps) => ps,
      Err(err) => bail!("Failed to parse response: {err}"),
    };

    if !page_response.page_props.server_response.data.records.is_empty() {
      self.page += 1;
    }

    Ok(page_response)
  }

  async fn new_session(&mut self) -> Result<()> {
    let args = ARGS.read().await.clone();
    let puppeteer = SecTrailPuppeteer::new(args.email.unwrap_or_default(), args.password.unwrap_or_default()).await?;
    let jar = Jar::default();
    for cookie in puppeteer.cookies {
      jar.add_cookie_str(cookie.to_cookie_str().as_str(), &self.base_url);
    }
    let client = Client::builder()
      .default_headers(puppeteer.headers)
      .cookie_store(true)
      .cookie_provider(Arc::from(jar))
      .build()?;
    let new_base = format!("https://securitytrails.com/_next/data/{}/list/", puppeteer.unique);

    self.client = client;
    self.expired = false;
    self.base_url = Url::from_str(&new_base)?;

    Ok(())
  }

  fn next_url(&self) -> Url {
    let data = self.data.as_ref();
    let page = self.page();

    let component = if let Ok(ip) = data.parse::<Ipv4Addr>() {
      format!("ip/{ip}.json?ip={ip}&page={page}")
    } else {
      format!("keyword/{data}.json?keyword={data}&page={page}")
    };

    self.base_url.join(component.as_str()).unwrap()
  }

  fn page(&self) -> usize {
    if self.page > 1 { self.page } else { 1 }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_sectrail_client() {
    let mut sectrail = SecTrailClient::new();
    let page1 = sectrail.get().await;

    assert!(page1.is_ok(), "{}", page1.unwrap_err());
    println!("{}", serde_json::to_string_pretty(page1.as_ref().unwrap()).unwrap());
    assert_eq!(page1.as_ref().unwrap().page_props.server_response.data.records.len(), 100);
  }
}
