use crate::span::Span;
use crossbeam_channel::Receiver;

pub struct Collector {
    receiver: Receiver<Vec<Span>>,
}

impl Collector {
    pub fn collect(&self) -> Vec<Span> {
        self.receiver.try_iter().flatten().collect()
    }
}

impl Collector {
    pub(crate) fn new(receiver: Receiver<Vec<Span>>) -> Self {
        Collector { receiver }
    }
}
