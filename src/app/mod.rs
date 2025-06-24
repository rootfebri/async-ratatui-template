pub use exports::*;
use tokio::sync::{mpsc, watch};

type WatchRx<T> = watch::Receiver<T>;
type WatchTx<T> = watch::Sender<T>;
type MpscRx<T> = mpsc::Receiver<T>;
type MpscTx<T> = mpsc::Sender<T>;

pub mod handler;

mod app_;
mod exports;
mod popup;
mod screen;
mod scroll_states;
mod state;
