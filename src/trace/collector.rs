use crate::cycle_to_realtime;
use crate::span::Span;
use crossbeam_channel::Receiver;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct Collector {
    receiver: Receiver<Vec<Span>>,
    closed: Arc<AtomicBool>,
}

impl Collector {
    /// Collects spans from traced routines.
    ///
    /// If passing `duration_threshold`, all spans will be reserved only when duration of the root
    /// span exceeds `duration_threshold`, otherwise only one span, the root span, will be returned.
    pub fn collect(self, duration_threshold: Option<Duration>) -> Vec<Span> {
        let spans: Vec<_> = self.receiver.try_iter().flatten().collect();
        self.closed.store(true, Ordering::SeqCst);
        if let Some(duration) = duration_threshold {
            if let Some(span) = spans.iter().find(|s| s.is_root()) {
                let duration_ns =
                    cycle_to_realtime(span.end_cycle).ns - cycle_to_realtime(span.begin_cycle).ns;
                if duration_ns < duration.as_nanos() as _ {
                    return vec![span.clone()];
                }
            }
        }
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
