use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

pub struct Statistic {
  cur: Arc<AtomicUsize>,
  max: Arc<AtomicUsize>,
}
