use std::path::PathBuf;

use clap::Parser;
use tokio::time::Interval;

#[derive(Parser, Debug, Clone, Default)]
#[command(about = "Ratatui Friend Async Template - Security Trails Edition", version, author = "Febri")]
pub struct AppArgs {
  /// Number of FPS for the app to run at (WARNING: Higher FPS may cause performance issues) Default is 0 (0 = Disabled and only render on event change)
  #[arg(long, default_value = "0")]
  pub fps: u8,

  /// Email address will be used by this app
  #[arg(short, long)]
  pub email: Option<String>,

  /// Password will be used by this app
  #[arg(short, long)]
  pub password: Option<String>,

  /// Headless will be used by this app
  #[arg(long, default_value = "false")]
  pub headless: bool,

  /// Username will be used by this app
  #[arg(short, long)]
  pub username: Option<String>,

  /// Input File, leave it if non urgen app use on entry (After launched can be changed)
  #[arg(short, long)]
  pub input: Option<PathBuf>,

  /// Output File, leave it if non urgen app use on entry (After launched can be changed)
  #[arg(short, long)]
  pub output: Option<PathBuf>,
}

impl AppArgs {
  pub fn create_fps_interval(&self) -> Interval {
    if self.fps == 0 {
      tokio::time::interval(tokio::time::Duration::from_secs(60)) // 60 seconds interval if FPS is 0
    } else {
      tokio::time::interval(tokio::time::Duration::from_millis(1000 / self.fps.max(5) as u64))
    }
  }
}
