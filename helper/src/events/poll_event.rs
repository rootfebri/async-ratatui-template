use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

use crossterm::event::{Event, EventStream};
use futures::{FutureExt, Stream, StreamExt};
use pin_project::pin_project;

#[derive(Default)]
pub struct PollEvent {
  inner: EventStream,
}

impl PollEvent {
  pub async fn read(&mut self) -> Result<Event>
  where
    Self: Unpin,
  {
    InnerPollEvent { stream: &mut self.inner }.await
  }

  pub async fn fuse_read(&mut self) -> Result<Event> {
    loop {
      if let Some(event) = self.inner.next().fuse().await {
        break event;
      } else {
        futures::task::noop_waker().wake();
      }
    }
  }
}

#[pin_project]
struct InnerPollEvent<'s> {
  #[pin]
  stream: &'s mut EventStream,
}

impl<'s> Future for InnerPollEvent<'s> {
  type Output = Result<Event>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match self.project().stream.poll_next(cx) {
      Poll::Ready(Some(event)) => Poll::Ready(event),
      Poll::Ready(None) | Poll::Pending => Poll::Pending,
    }
  }
}
