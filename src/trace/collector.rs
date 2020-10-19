use crate::span::Span;
use crossbeam_channel::Receiver;
use std::collections::HashMap;
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
        Self::remove_spawn_spans(spans)
    }
}

impl Collector {
    fn remove_spawn_spans(mut spans: Vec<Span>) -> Vec<Span> {
        let mut spawn_spans = HashMap::new();
        for span in &spans {
            if span._is_spawn_span {
                spawn_spans.insert(span.id, span.parent_id);
            }
        }

        for span in &mut spans {
            if let Some(p) = spawn_spans.get(&span.parent_id) {
                span.parent_id = *p;
            }
        }

        spans.into_iter().filter(|s| !s._is_spawn_span).collect()
    }
}

impl Collector {
    pub(crate) fn new(receiver: Receiver<Vec<Span>>, closed: Arc<AtomicBool>) -> Self {
        Collector { receiver, closed }
    }
}
