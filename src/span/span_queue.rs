use crate::collections::queue::FixedIndexQueue;
use crate::span::cycle::Clock;
use crate::span::span_id::{IdGenerator, SpanId};
use crate::span::{ExternalSpan, Span};

pub struct SpanQueue<IdGenerator, Clock> {
    span_queue: FixedIndexQueue<Span>,
    next_parent_id: SpanId,
    id_generator: IdGenerator,
    clock: Clock,
}

impl<IG: IdGenerator, C: Clock> SpanQueue<IG, C> {
    pub fn new(id_generator: IG, clock: C) -> Self {
        Self {
            span_queue: FixedIndexQueue::new(),
            next_parent_id: SpanId::new(0),
            id_generator,
            clock,
        }
    }

    pub fn start_span(&mut self, event: &'static str) -> Finisher {
        let s = self.gen_span(self.next_parent_id, event);
        self.next_parent_id = s.id();
        let index = self.push_span(s);
        Finisher { index }
    }

    pub fn finish_span(&mut self, finisher: Finisher) {
        if !self.span_queue.idx_is_valid(finisher.index) {
            return;
        }

        let descendant_count = self.count_to_last(finisher.index);
        let span = &mut self.span_queue[finisher.index];
        span.end_with(self.clock.now(), descendant_count);

        self.next_parent_id = span.parent_id();
    }

    pub fn start_external_span(
        &mut self,
        placeholder_event: &'static str,
        event: &'static str,
    ) -> ExternalSpan {
        let mut s = self.gen_span(self.next_parent_id, placeholder_event);
        s.end_cycles = s.begin_cycles;
        let es_parent = s.id();
        self.push_span(s);
        self.gen_external_span(es_parent, event)
    }

    pub fn start_root_external_span(&mut self, event: &'static str) -> ExternalSpan {
        self.gen_external_span(SpanId::new(0), event)
    }

    pub fn finish_external_span(&self, external_span: &ExternalSpan) -> Span {
        external_span.to_span(self.clock.now())
    }

    pub fn next_index(&self) -> usize {
        self.span_queue.next_index()
    }

    pub fn clear(&mut self) {
        self.span_queue.clear();
        self.next_parent_id = SpanId::new(0);
    }

    pub fn remove_before(&mut self, index: usize) {
        self.span_queue.remove_before(index);
    }

    pub fn iter_skip_to(&self, index: usize) -> impl Iterator<Item = &Span> {
        self.span_queue.iter_skip_to(index)
    }
}

impl<IG: IdGenerator, C: Clock> SpanQueue<IG, C> {
    #[inline]
    fn gen_span(&self, parent_id: SpanId, event: &'static str) -> Span {
        Span::begin_with(
            self.id_generator.next_id(),
            parent_id,
            self.clock.now(),
            event,
        )
    }

    #[inline]
    fn gen_external_span(&self, parent_id: SpanId, event: &'static str) -> ExternalSpan {
        ExternalSpan::new(
            self.id_generator.next_id(),
            parent_id,
            self.clock.now(),
            event,
        )
    }

    #[inline]
    fn push_span(&mut self, span: Span) -> usize {
        self.span_queue.push_back(span)
    }

    fn count_to_last(&self, index: usize) -> usize {
        let next_index = self.span_queue.next_index();
        next_index.wrapping_sub(index) - 1
    }
}

pub struct Finisher {
    pub(self) index: usize,
}
