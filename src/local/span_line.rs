use crate::local::registry::{Listener, Registry};
use crate::span::cycle::{Clock, TempClock};
use crate::span::span_id::{IdGenerator, SpanId, TempIdGenerator};
use crate::span::span_queue::{Finisher, SpanQueue};
use crate::span::{ExternalSpan, Span};
use crate::trace::acquirer::AcquirerGroup;
use slab::Slab;
use std::cell::UnsafeCell;
use std::sync::Arc;

thread_local! {
    pub(super) static SPAN_LINE: UnsafeCell<SpanLine<TempIdGenerator, TempClock>> = UnsafeCell::new(SpanLine::new(TempIdGenerator, TempClock));
}

pub struct SpanLine<IdGenerator, Clock> {
    span_queue: SpanQueue<IdGenerator, Clock>,
    registry: Registry,
    local_acquirer_groups: Slab<Arc<AcquirerGroup>>,
}

impl<IG: IdGenerator, C: Clock> SpanLine<IG, C> {
    pub fn new(id_generator: IG, clock: C) -> Self {
        Self {
            span_queue: SpanQueue::new(id_generator, clock),
            registry: Registry::default(),
            local_acquirer_groups: Slab::default(),
        }
    }

    pub fn start_span(&mut self, event: &'static str) -> Option<Finisher> {
        if self.registry.is_empty() {
            return None;
        }

        Some(self.span_queue.start_span(event))
    }

    pub fn finish_span(&mut self, finisher: Finisher) {
        self.span_queue.finish_span(finisher);
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

    pub fn spans_from(&self, listener: Listener) -> impl Iterator<Item = Span> + '_ {
        Iter::new(self.span_queue.iter_skip_to(listener.queue_index))
    }

    pub fn unregister(&mut self, listener: Listener) -> Arc<AcquirerGroup> {
        let acg = self.local_acquirer_groups.remove(listener.slab_index);
        self.registry.unregister(listener);
        self.gc();
        acg
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
}

impl<IG: IdGenerator, C: Clock> SpanLine<IG, C> {
    fn gc(&mut self) {
        if let Some(l) = self.registry.earliest_listener() {
            self.span_queue.remove_before(l.queue_index);
        } else {
            self.span_queue.clear();
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

pub struct Iter<'a, I: Iterator<Item = &'a Span>> {
    raw_iter: I,
    remaining_descendants: usize,
}

impl<'a, I: Iterator<Item = &'a Span>> Iter<'a, I> {
    pub fn new(raw_iter: I) -> Self {
        Self {
            raw_iter,
            remaining_descendants: 0,
        }
    }
}

impl<'a, I: Iterator<Item = &'a Span>> Iterator for Iter<'a, I> {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_descendants > 0 {
            self.remaining_descendants -= 1;
            return self.raw_iter.next().cloned();
        }

        while let Some(span) = self.raw_iter.next() {
            // skip non-finished span
            if span.end_cycles().is_zero() {
                continue;
            }

            self.remaining_descendants = span._descendant_count;

            // set as a root span
            let mut span = *span;
            span.set_parent_id(SpanId::new(0));

            return Some(span);
        }

        None
    }
}
