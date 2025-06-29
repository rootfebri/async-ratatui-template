use std::io::stdout;
use std::process::Stdio;

use anyhow::{Result, bail};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use tokio::process::Command;

use crate::args::AppArgs;

#[derive(Debug, Clone)]
pub struct SecTrailPuppeteer {
  pub success: bool,
  pub message: String,
  pub cookies: Vec<Cookie>,
  pub headers: HeaderMap,
  pub unique: String,
}

impl SecTrailPuppeteer {
  pub async fn new(args: &AppArgs) -> Result<Self> {
    setup_bun().await?;

    let cmd = Command::new("bun")
      .args([
        String::from("run"),
        String::from("index.ts"),
        args.email.clone().unwrap_or_default(),
        args.password.clone().unwrap_or_default(),
      ])
      .kill_on_drop(true)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .await?;

    if cmd.status.success() {
      serde_json::from_str(String::from_utf8_lossy(&cmd.stdout).as_ref()).map_err(anyhow::Error::from)
    } else {
      bail!(String::from_utf8_lossy(&cmd.stderr).into_owned())
    }
  }
}

pub async fn setup_bun() -> Result<()> {
  let which = if cfg!(windows) { "where.exe" } else { "which" };
  let bun_cmd = Command::new(which).stdout(stdout()).stderr(stdout()).args(["bun"]).output().await?;
  if bun_cmd.status.success() {
    return Ok(());
  }

  if cfg!(windows) {
    Command::new("powershell")
      .args(["-c", "irm bun.sh/install.ps1 | iex"])
      .stdout(stdout())
      .stderr(stdout())
      .output()
      .await?;
  } else {
    Command::new("bash")
      .args(["-c", "curl -fsSL https://bun.sh/install | bash"])
      .stdout(stdout())
      .stderr(stdout())
      .output()
      .await?;
  }

  Command::new("bun").args(["install"]).output().await?;

  Ok(())
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

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_puppeteer() {
    let args = AppArgs {
      email: Some(String::from("dagelanfl@pakde.io")),
      password: Some(String::from("Bocahkosong@588")),
      ..Default::default()
    };

    let puppeteer = SecTrailPuppeteer::new(&args).await;
    assert!(puppeteer.is_ok(), "{}", puppeteer.unwrap_err());
  }
}
