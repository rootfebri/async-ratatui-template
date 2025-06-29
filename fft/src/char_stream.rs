use std::path::Path;
use std::{io, str};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub struct CharStream {
  file: File,
  buf: Vec<u8>,
}

impl CharStream {
  pub async fn new(path: impl AsRef<Path>) -> io::Result<Self> {
    let file = File::open(path).await?;
    Ok(Self {
      file,
      buf: Vec::with_capacity(4),
    })
  }

  pub async fn next_char(&mut self) -> io::Result<Option<char>> {
    loop {
      if let Ok(s) = str::from_utf8(&self.buf) {
        if let Some(c) = s.chars().next() {
          let char_len = c.len_utf8();
          self.buf.drain(..char_len);
          return Ok(Some(c));
        }
      }

      if self.buf.len() >= 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8 sequence"));
      }

      let mut b = [0; 1];
      let n = self.file.read(&mut b).await?;

      if n == 0 {
        return if self.buf.is_empty() {
          Ok(None)
        } else {
          Err(io::Error::new(io::ErrorKind::InvalidData, "incomplete char at EOF"))
        };
      }
      self.buf.push(b[0]);
    }
  }
}
