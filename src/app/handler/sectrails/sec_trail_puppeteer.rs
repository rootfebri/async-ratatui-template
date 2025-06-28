use std::collections::BTreeMap;
use std::process::Stdio;

use anyhow::{Result, bail};
use serde::Deserialize;
use tokio::process::Command;

use crate::args::AppArgs;

#[derive(Debug, Clone, Deserialize)]
pub struct SecTrailPuppeteer {
  #[serde(default)]
  pub success: bool,
  #[serde(default)]
  pub message: String,
  #[serde(default)]
  pub cookies: Vec<Cookie>,
  #[serde(default)]
  pub headers: BTreeMap<String, String>,
  #[serde(rename = "userAgent")]
  pub user_agent: String,
}

impl SecTrailPuppeteer {
  pub async fn new(args: AppArgs) -> Result<Self> {
    let cmd = Command::new("bun")
      .args([
        String::from("run"),
        String::from("index.ts"),
        args.email.unwrap_or_default(),
        args.password.unwrap_or_default(),
        String::from(if args.headless { "headless" } else { "" }),
      ])
      .kill_on_drop(true)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .await?;

    println!("{}", String::from_utf8_lossy(&cmd.stderr));
    println!("{}", String::from_utf8_lossy(&cmd.stdout));
    println!("{}", cmd.status);
    if cmd.status.code().is_some_and(|code| code > 1) {
      bail!(String::from_utf8_lossy(&cmd.stderr).into_owned())
    }
    Ok(serde_json::from_str(String::from_utf8_lossy(&cmd.stdout).as_ref())?)
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cookie {
  /// Cookie name.
  pub name: String,
  /// Cookie value.
  pub value: String,
  /// Cookie domain.
  pub domain: String,
  /// Cookie path.
  pub path: String,
  /// Cookie expiration date as the number of seconds since the UNIX epoch. Set to `-1` for session cookies.
  pub expires: f64,
  /// Cookie size.
  pub size: usize,
  /// True if cookie is http-only.
  #[serde(default, skip_serializing_if = "Option::None")]
  pub http_only: Option<bool>,
  /// True if cookie is secure.
  pub secure: bool,
  /// True in case of session cookie.
  pub session: bool,
  /// Cookie SameSite type.
  pub same_site: Option<SameSite>,
  /// Cookie Priority. Supported only in Chrome.
  pub priority: Option<Priority>,
  /// True if cookie is SameParty. Supported only in Chrome.
  pub same_party: Option<bool>,
  /// Cookie source scheme type. Supported only in Chrome.
  pub source_scheme: Option<SourceScheme>,
  /// Cookie partition key. In Chrome, it is the top-level site the partitioned cookie is available in. In Firefox, it matches the source origin.
  pub partition_key: Option<PartitionKey>,
  /// True if cookie partition key is opaque. Supported only in Chrome.
  pub partition_key_opaque: Option<bool>,

  #[serde(skip, default)]
  vectorized: Vec<char>,
}

impl Cookie {
  pub fn to_cookie_str(&self) -> String {
    let mut cookie_str = format!("{}={}", self.name, self.value);

    if !self.path.is_empty() {
      cookie_str.push_str(&format!("; Path={}", self.path));
    }

    if !self.domain.is_empty() {
      cookie_str.push_str(&format!("; Domain={}", self.domain));
    }

    if self.secure {
      cookie_str.push_str("; Secure");
    }

    if self.http_only.unwrap_or(false) {
      cookie_str.push_str("; HttpOnly");
    }

    if let Some(same_site) = &self.same_site {
      let samesite_str = match same_site {
        SameSite::Strict => "Strict",
        SameSite::Lax => "Lax",
        SameSite::None => "None",
      };
      cookie_str.push_str(&format!("; SameSite={samesite_str}"));
    }

    cookie_str
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionKey {
  pub source_origin: String,
  pub has_cross_site_ancestor: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum SameSite {
  Strict,
  Lax,
  None,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Priority {
  Low,
  Medium,
  High,
}

#[derive(Debug, Clone, Deserialize)]
pub enum SourceScheme {
  Unset,
  NonSecure,
  Secure,
}
