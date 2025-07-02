use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, bail};
use reqwest::cookie::Jar;
use reqwest::{Client, Url};

use super::*;
use crate::SafeRefStr;
use crate::app::handler::sectrails::jsons::PageResponse;

#[derive(Debug)]
pub struct SecTrailClient {
  page: usize,
  data: Arc<str>,
  client: Client,
  expired: bool,
  base_url: Url,
  email: SafeRefStr,
  password: SafeRefStr,
}
impl Default for SecTrailClient {
  fn default() -> Self {
    Self::new(Default::default(), Default::default())
  }
}
impl SecTrailClient {
  const BASE_URL: &'static str = "https://securitytrails.com/_next/data/";
  pub fn new(email: SafeRefStr, password: SafeRefStr) -> Self {
    let base_url = Url::parse(Self::BASE_URL).unwrap();

    Self {
      page: 1,
      data: Arc::default(),
      client: Default::default(),
      expired: true,
      base_url,
      email,
      password,
    }
  }

  pub fn new_sequence(&mut self, data: impl Into<Arc<str>>) {
    self.data = data.into();
    self.page = 1;
  }

  pub async fn get(&mut self) -> Result<PageResponse> {
    if self.expired {
      self.new_session().await?;
    }

    let url = self.next_url();
    let response = match self.client.get(url).send().await {
      Ok(response) => response,
      Err(error) => {
        if error.is_request() {
          self.expired = true;
        }
        bail!(error);
      }
    };

    let body = response.bytes().await?;

    #[cfg(test)]
    {
      eprintln!("Response: {}", String::from_utf8_lossy(&body));
    }

    let page_response: PageResponse = serde_json::from_slice(&body)?;

    if !page_response.page_props.server_response.data.records.is_empty() {
      self.page += 1;
    }

    Ok(page_response)
  }

  async fn new_session(&mut self) -> Result<()> {
    let puppeteer = SecTrailPuppeteer::new(self.email.read().await, self.password.read().await).await?;
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
  use crate::new_safe_str;

  #[tokio::test]
  async fn test_sectrail_client() {
    let email = new_safe_str("dagelanfl@pakde.io");
    let password = new_safe_str("Bocahkosong@588");

    let mut sectrail = SecTrailClient::new(email, password);
    let page1 = sectrail.get().await;

    assert!(page1.is_ok(), "{}", page1.unwrap_err());
    println!("{}", serde_json::to_string_pretty(page1.as_ref().unwrap()).unwrap());
    assert_eq!(page1.as_ref().unwrap().page_props.server_response.data.records.len(), 100);
  }
}
