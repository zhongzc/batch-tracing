use crate::local::acquirer_group::submit_task_span;
use crate::span::{ExternalSpan, Span};

#[derive(Clone, Debug)]
pub struct AcquirerGroup {
    /// A span represents task processing
    task_span: ExternalSpan,
    acquirers: Vec<Acquirer>,
}

impl AcquirerGroup {
    pub fn new(span: ExternalSpan, acquirers: Vec<Acquirer>) -> Self {
        AcquirerGroup {
            task_span: span,
            acquirers,
        }
    }

    pub fn combine<'a, I: Iterator<Item = &'a AcquirerGroup>>(
        iter: I,
        external_span: ExternalSpan,
    ) -> Self {
        let acquirers = iter
            .map(|s| s.acquirers.clone())
            .flatten()
            .collect::<Vec<_>>();
        Self {
            task_span: external_span,
            acquirers,
        }
    }

    pub fn submit(&self, mut spans: Vec<Span>) {
        self.modify_root_spans(&mut spans);
        self.submit_to_acquirers(spans);
    }

    pub fn submit_task_span(&self, task_span: Span) {
        self.submit_to_acquirers(vec![task_span]);
    }
}

impl AcquirerGroup {
    fn modify_root_spans(&self, spans: &mut [Span]) {
        for span in spans {
            if span.is_root() {
                span.set_parent_id(self.task_span.id());
            }
        }
    }

    fn submit_to_acquirers(&self, spans: Vec<Span>) {
        if self.acquirers.len() == 1 {
            self.acquirers[0].submit(spans);
        } else {
            for acq in &self.acquirers {
                acq.submit(spans.clone());
            }
        }
    }
}

impl Drop for AcquirerGroup {
    fn drop(&mut self) {
        submit_task_span(self, &self.task_span)
    }
}

#[derive(Clone, Debug)]
pub struct Acquirer {}

impl Acquirer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn submit(&self, spans: Vec<Span>) {}
}
