pub mod cycle;
pub mod span_id;
pub mod span_queue;

use crate::span::cycle::Cycle;
use crate::span::span_id::SpanId;

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub id: SpanId,
    pub parent_id: SpanId,
    pub begin_cycles: Cycle,
    pub event: &'static str,

    // post processing will write this
    pub end_cycles: Cycle,

    // for local queue implementation
    pub(crate) _descendant_count: usize,

    // a tag
    pub(crate) _is_spawn_span: bool,
}

impl Span {
    pub(crate) fn begin_with(
        id: SpanId,
        parent_id: SpanId,
        begin_cycles: Cycle,
        event: &'static str,
    ) -> Self {
        Span {
            id,
            parent_id,
            begin_cycles,
            event,
            end_cycles: Cycle::default(),
            _descendant_count: 0,
            _is_spawn_span: false,
        }
    }

    pub(crate) fn end_with(&mut self, end_cycles: Cycle, descendant_count: usize) {
        self.end_cycles = end_cycles;
        self._descendant_count = descendant_count;
    }

    pub(crate) fn is_root(&self) -> bool {
        self.parent_id == SpanId::new(0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ExternalSpan {
    pub id: SpanId,
    pub parent_id: SpanId,
    pub begin_cycles: Cycle,
    pub event: &'static str,
}

impl ExternalSpan {
    pub fn new(id: SpanId, parent_id: SpanId, begin_cycles: Cycle, event: &'static str) -> Self {
        ExternalSpan {
            id,
            parent_id,
            begin_cycles,
            event,
        }
    }

    pub fn to_span(&self, end_cycles: Cycle) -> Span {
        Span {
            id: self.id,
            parent_id: self.parent_id,
            begin_cycles: self.begin_cycles,
            event: self.event,
            end_cycles,
            _descendant_count: 0,
            _is_spawn_span: false,
        }
    }
}
