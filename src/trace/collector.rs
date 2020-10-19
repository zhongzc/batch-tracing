use crate::span::Span;
use crossbeam_channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Collector {
    receiver: Receiver<Vec<Span>>,
    closed: Arc<AtomicBool>,
}

impl Collector {
    pub fn collect(self) -> Vec<Span> {
        let spans = self.receiver.try_iter().flatten().collect();
        self.closed.store(true, Ordering::SeqCst);
        spans
    }
}

impl Collector {
    pub(crate) fn new(receiver: Receiver<Vec<Span>>, closed: Arc<AtomicBool>) -> Self {
        Collector { receiver, closed }
    }
}
