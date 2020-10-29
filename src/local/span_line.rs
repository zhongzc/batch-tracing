use crate::local::registry::{Listener, Registry};
use crate::span::span_id::SpanId;
use crate::span::span_queue::{SpanHandle, SpanQueue};
use crate::span::{ExternalSpan, Span};
use crate::trace::acquirer::AcquirerGroup;
use slab::Slab;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    pub(super) static SPAN_LINE: RefCell<SpanLine> = RefCell::new(SpanLine::new());
}

pub struct SpanLine {
    span_queue: SpanQueue,
    registry: Registry,
    local_acquirer_groups: Slab<Arc<AcquirerGroup>>,
}

impl SpanLine {
    pub fn new() -> Self {
        Self {
            span_queue: SpanQueue::new(),
            registry: Registry::default(),
            local_acquirer_groups: Slab::default(),
        }
    }

    #[inline]
    pub fn start_span(&mut self, event: &'static str) -> Option<SpanHandle> {
        if self.registry.is_empty() {
            return None;
        }

        Some(self.span_queue.start_span(event))
    }

    #[inline]
    pub fn finish_span(&mut self, span_handle: SpanHandle) {
        self.span_queue.finish_span(span_handle);
    }

    pub fn start_root_external_span(&mut self, event: &'static str) -> ExternalSpan {
        self.span_queue.start_root_external_span(event)
    }

    pub fn finish_external_span(&self, external_span: &ExternalSpan) -> Span {
        self.span_queue.finish_external_span(external_span)
    }

    pub fn register_now(&mut self, acquirer_group: Arc<AcquirerGroup>) -> Listener {
        let slab_idx = self.local_acquirer_groups.insert(acquirer_group);
        let l = Listener::new(self.span_queue.next_index(), slab_idx);
        self.registry.register(l);
        l
    }

    pub fn unregister_and_collect(
        &mut self,
        listener: Listener,
    ) -> (Arc<AcquirerGroup>, Vec<Span>) {


        // let spans = if self.registry.is_empty() {
        //     Iter::new(self.span_queue.iter_skip_to(listener.queue_index)).collect()
        // } else {
        let spans = Iter::new(self.span_queue.iter_ref_skip_to(listener.queue_index)).collect();
        // };

        let acg = self.local_acquirer_groups.remove(listener.slab_index);
        self.registry.unregister(listener);

        self.gc();

        (acg, spans)
    }

    /// Return `None` if there're no registered acquirers, or all acquirers
    /// combined into one group.
    pub fn registered_acquirer_group(&mut self, event: &'static str) -> Option<AcquirerGroup> {
        match self.start_external_span("<spawn>", event) {
            None => None,
            Some(es) => Some(AcquirerGroup::combine(
                self.local_acquirer_groups.iter().map(|s| s.1.as_ref()),
                es,
            )),
        }
    }

    #[inline]
    pub fn add_properties<I: IntoIterator<Item = (&'static str, String)>, F: FnOnce() -> I>(
        &mut self,
        span_handle: &SpanHandle,
        properties: F,
    ) {
        self.span_queue.add_properties(span_handle, properties);
    }

    #[inline]
    pub fn add_property<F: FnOnce() -> (&'static str, String)>(
        &mut self,
        span_handle: &SpanHandle,
        property: F,
    ) {
        self.span_queue.add_property(span_handle, property);
    }
}

impl SpanLine {
    fn gc(&mut self) {
        if let Some(l) = self.registry.earliest_listener() {
            self.span_queue.remove_before(l.queue_index);
        }
    }

    fn start_external_span(
        &mut self,
        placeholder_event: &'static str,
        event: &'static str,
    ) -> Option<ExternalSpan> {
        if self.registry.is_empty() {
            return None;
        }

        Some(
            self.span_queue
                .start_external_span(placeholder_event, event),
        )
    }
}

/// An iterator that collecting proper completed spans from the queue iterator.
pub struct Iter<S: Into<Span> + AsRef<Span>, I: Iterator<Item = S>> {
    raw_iter: I,
    remaining_descendants: usize,
}

impl<S: Into<Span> + AsRef<Span>, I: Iterator<Item = S>> Iter<S, I> {
    pub fn new(raw_iter: I) -> Self {
        Self {
            raw_iter,
            remaining_descendants: 0,
        }
    }
}

impl<S: Into<Span> + AsRef<Span>, I: Iterator<Item = S>> Iterator for Iter<S, I> {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_descendants > 0 {
            self.remaining_descendants -= 1;
            return self.raw_iter.next().map(Into::into);
        }

        while let Some(span) = self.raw_iter.next() {
            // skip non-finished span
            let span = span.as_ref();
            if span.end_cycle.is_zero() {
                continue;
            }

            self.remaining_descendants = span._descendant_count;

            // set as a root span
            let mut span: Span = span.into();
            span.parent_id = SpanId::new(0);

            return Some(span);
        }

        None
    }
}
