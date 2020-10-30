use crate::span::cycle::DefaultClock;
use crate::span::{ScopeSpan, Span};
use crossbeam_channel::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Acquirer {
    sender: Arc<Sender<Vec<Span>>>,
    closed: Arc<AtomicBool>,
}

impl Acquirer {
    pub fn new(sender: Arc<Sender<Vec<Span>>>, closed: Arc<AtomicBool>) -> Self {
        Acquirer { sender, closed }
    }

    pub fn submit(&self, spans: Vec<Span>) {
        self.sender.send(spans).ok();
    }

    pub fn is_shutdown(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Debug)]
pub struct AcquirerGroup {
    /// A span represents task processing
    scope_span: ScopeSpan,
    acquirers: Vec<Acquirer>,
}

impl AcquirerGroup {
    pub fn new(span: ScopeSpan, acquirers: Vec<Acquirer>) -> Self {
        debug_assert!(!acquirers.is_empty());

        AcquirerGroup {
            scope_span: span,
            acquirers,
        }
    }

    pub fn combine<'a, I: Iterator<Item = &'a AcquirerGroup>>(
        iter: I,
        external_span: ScopeSpan,
    ) -> Self {
        let acquirers = iter
            .map(|s| {
                s.acquirers.iter().filter_map(|acq| {
                    if acq.is_shutdown() {
                        None
                    } else {
                        Some(acq.clone())
                    }
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        debug_assert!(!acquirers.is_empty());

        Self {
            scope_span: external_span,
            acquirers,
        }
    }

    pub fn submit(&self, mut spans: Vec<Span>) {
        self.modify_root_spans(&mut spans);
        self.submit_to_acquirers(spans);
    }

    pub fn submit_scope_span(&self, task_span: Span) {
        self.submit_to_acquirers(vec![task_span]);
    }
}

impl AcquirerGroup {
    fn modify_root_spans(&self, spans: &mut [Span]) {
        for span in spans {
            if span.is_root() {
                span.parent_id = self.scope_span.id;
            }
        }
    }

    fn submit_to_acquirers(&self, spans: Vec<Span>) {
        // save one clone
        for acq in self.acquirers.iter().skip(1) {
            acq.submit(spans.clone());
        }
        if let Some(acq) = self.acquirers.first() {
            acq.submit(spans);
        }
    }
}

impl Drop for AcquirerGroup {
    fn drop(&mut self) {
        self.submit_scope_span(self.scope_span.to_span(DefaultClock::now()));
    }
}
