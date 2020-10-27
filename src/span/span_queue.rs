use crate::collections::queue::FixedIndexQueue;
use crate::span::cycle::{Clock, Cycle, Realtime};
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

    #[inline]
    pub fn start_span(&mut self, event: &'static str) -> SpanHandle {
        let s = self.gen_span(self.next_parent_id, event);
        self.next_parent_id = s.id;
        let index = self.push_span(s);
        SpanHandle { index }
    }

    #[inline]
    pub fn finish_span(&mut self, span_handle: SpanHandle) {
        debug_assert!(self.span_queue.idx_is_valid(span_handle.index));

        let descendant_count = self.count_to_last(span_handle.index);
        let span = &mut self.span_queue[span_handle.index];
        span.end_with(self.clock.now(), descendant_count);

        self.next_parent_id = span.parent_id;
    }

    #[inline]
    pub fn add_properties<I: IntoIterator<Item = (&'static str, String)>, F: FnOnce() -> I>(
        &mut self,
        span_handle: &SpanHandle,
        properties: F,
    ) {
        debug_assert!(self.span_queue.idx_is_valid(span_handle.index));

        let span = &mut self.span_queue[span_handle.index];
        span.properties.extend(properties());
    }

    #[inline]
    pub fn add_property<F: FnOnce() -> (&'static str, String)>(
        &mut self,
        span_handle: &SpanHandle,
        property: F,
    ) {
        debug_assert!(self.span_queue.idx_is_valid(span_handle.index));

        let span = &mut self.span_queue[span_handle.index];
        span.properties.push(property());
    }

    #[inline]
    pub fn start_external_span(
        &mut self,
        placeholder_event: &'static str,
        event: &'static str,
    ) -> ExternalSpan {
        // add a spawn span for indirectly linking to the external span
        let mut s = self.gen_span(self.next_parent_id, placeholder_event);
        let cycle = s.begin_cycle;
        s.end_cycle = cycle;
        s._is_spawn_span = true;
        let es_parent = s.id;
        self.push_span(s);

        self.gen_external_span(es_parent, event, cycle)
    }

    #[inline]
    pub fn start_root_external_span(&mut self, event: &'static str) -> ExternalSpan {
        self.gen_external_span(SpanId::new(0), event, self.clock.now())
    }

    #[inline]
    pub fn finish_external_span(&self, external_span: &ExternalSpan) -> Span {
        external_span.to_span(self.clock.now())
    }

    #[inline]
    pub fn next_index(&self) -> usize {
        self.span_queue.next_index()
    }

    #[inline]
    pub fn remove_before(&mut self, index: usize) {
        self.span_queue.remove_before(index);
    }

    #[inline]
    pub fn iter_skip_to(&self, index: usize) -> impl Iterator<Item = &Span> {
        self.span_queue.iter_ref_skip_to(index)
    }

    #[inline]
    pub fn into_iter_skip_to(&mut self, index: usize) -> impl Iterator<Item = Span> {
        self.span_queue.iter_skip_to(index)
    }

    #[inline]
    pub fn cycle_to_realtime(&self, cycle: Cycle) -> Realtime {
        self.clock.cycle_to_realtime(cycle)
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
    fn gen_external_span(
        &self,
        parent_id: SpanId,
        event: &'static str,
        begin_cycle: Cycle,
    ) -> ExternalSpan {
        ExternalSpan::new(self.id_generator.next_id(), parent_id, begin_cycle, event)
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

pub struct SpanHandle {
    pub(self) index: usize,
}
