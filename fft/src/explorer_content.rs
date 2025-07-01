use devicons::FileIcon;
use ratatui::prelude::{Color, Line, Span};
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExplorerContent {
  Dir { path: Arc<Path> },
  File { path: Arc<Path>, content: Arc<[u8]> },
}

impl ExplorerContent {
  pub fn filename(&self) -> Cow<str> {
    match *self {
      ExplorerContent::Dir { ref path, .. } | ExplorerContent::File { ref path, .. } => {
        if let Some(filename) = path.file_name() {
          return filename.to_string_lossy();
        }
        path.to_string_lossy()
      }
    }
  }

  pub fn icon(&self) -> Span<'static> {
    let fileicon = FileIcon::from(self.as_path());
    let color = Color::from_str(fileicon.color).unwrap_or(Color::Reset);
    let icon = fileicon.icon.to_string();

    Span::raw(icon).fg(color)
  }

  pub fn apply_colors(&self, input: &str) -> Line<'static> {
    let icon = self.icon();
    if input.is_empty() {
      let mut line = Line::default();
      line.push_span(icon);
      line.push_span(Span::raw(" "));
      line.push_span(Span::raw(self.filename().to_string()).white());

      return line;
    };

    let mut spans: Vec<Span> = vec![icon, Span::raw(" ")];

    let path_str = self.filename();
    let input_chars: Vec<_> = input.chars().collect();

    for chr in path_str.chars() {
      let span = Span::raw(chr.to_string());

      if input_chars.contains(&chr) {
        spans.push(span.fg(Color::Rgb(36, 132, 96)));
      } else {
        spans.push(span.fg(Color::White));
      }
    }

    spans.into()
  }

  pub fn is_dir(&self) -> bool {
    matches!(self, Self::Dir { .. })
  }
  pub fn is_file(&self) -> bool {
    matches!(self, Self::File { .. })
  }

  pub fn as_path(&self) -> &Path {
    match *self {
      ExplorerContent::Dir { ref path, .. } | ExplorerContent::File { ref path, .. } => path.as_ref(),
    }
  }

  pub fn content_lines(&self) -> Vec<Line> {
    std::thread::sleep(std::time::Duration::from_millis(1));

    match *self {
      ExplorerContent::Dir { .. } => vec![],
      ExplorerContent::File { ref content, .. } => content.split(|b| b == &10).map(String::from_utf8_lossy).map(Line::from).collect(),
    }
  }

  pub fn as_preview(&self) -> Paragraph {
    Paragraph::new(self.content_lines()).scroll((0, 0))
  }

  pub async fn async_new(path: impl AsRef<Path>) -> Self {
    let path = path.as_ref();
    if path.is_file() {
      let bytes = match File::open(path).await {
        Ok(file) => {
          let mut bufread = BufReader::new(file);
          if let Ok(metadata) = bufread.get_ref().metadata().await
            && metadata.len() <= crate::explorer::MAX_CONTENT_SIZE as u64
          {
            let mut vecread = vec![0u8; metadata.len() as usize];

            if let Err(err) = bufread.read_to_end(&mut vecread).await {
              let error = err.kind().to_string();
              if let Some(null) = vecread.iter().position(|b| b == &0) {
                vecread.truncate(null + 1);
              } else {
                vecread.extend_from_slice(error.as_bytes());
                if vecread.len() > crate::explorer::MAX_CONTENT_SIZE {
                  vecread.truncate(crate::explorer::MAX_CONTENT_SIZE);
                }
              }
            }

            vecread
          } else {
            let mut vectoread = vec![0u8; crate::explorer::MAX_CONTENT_SIZE];

            match bufread.read_exact(&mut vectoread).await {
              Ok(_) => vectoread,
              Err(err) => {
                if let std::io::ErrorKind::UnexpectedEof = err.kind() {
                  if let Some(null) = vectoread.iter().position(|b| b == &0) {
                    vectoread.truncate(null);
                  }
                  vectoread
                } else {
                  err.to_string().as_bytes().to_vec()
                }
              }
            }
          }
        }
        Err(err) => err.to_string().as_bytes().to_vec(),
      };

      Self::File {
        path: Arc::from(path),
        content: Arc::from(bytes),
      }
    } else {
      Self::Dir { path: Arc::from(path) }
    }
  }
}

impl PartialEq for ExplorerContent {
  fn eq(&self, other: &Self) -> bool {
    self.as_path() == other.as_path()
  }
}

impl PartialOrd for ExplorerContent {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.as_path().cmp(other.as_path()))
  }
}

impl Ord for ExplorerContent {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}

impl Eq for ExplorerContent {}
