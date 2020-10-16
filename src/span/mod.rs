pub mod cycle;
pub mod span_id;
pub mod span_queue;

use crate::span::cycle::Cycles;
use crate::span::span_id::SpanId;

#[derive(Copy, Clone, Debug)]
pub struct Span {
    id: SpanId,
    parent_id: SpanId,
    begin_cycles: Cycles,
    event: &'static str,

    // post processing will write this
    end_cycles: Cycles,

    // for local queue implementation
    pub(crate) _descendant_count: usize,
}

impl Span {
    pub fn id(&self) -> SpanId {
        self.id
    }
    pub fn set_id(&mut self, id: SpanId) {
        self.id = id;
    }
    pub fn parent_id(&self) -> SpanId {
        self.parent_id
    }
    pub fn set_parent_id(&mut self, parent_id: SpanId) {
        self.parent_id = parent_id;
    }
    pub fn begin_cycles(&self) -> Cycles {
        self.begin_cycles
    }
    pub fn set_begin_cycles(&mut self, begin_cycles: Cycles) {
        self.begin_cycles = begin_cycles;
    }
    pub fn event(&self) -> &'static str {
        self.event
    }
    pub fn set_event(&mut self, event: &'static str) {
        self.event = event;
    }
    pub fn end_cycles(&self) -> Cycles {
        self.end_cycles
    }
    pub fn set_end_cycles(&mut self, end_cycles: Cycles) {
        self.end_cycles = end_cycles;
    }
}

impl Span {
    pub(crate) fn begin_with(
        id: SpanId,
        parent_id: SpanId,
        begin_cycles: Cycles,
        event: &'static str,
    ) -> Self {
        Span {
            id,
            parent_id,
            begin_cycles,
            event,
            end_cycles: Cycles::default(),
            _descendant_count: 0,
        }
    }

    pub(crate) fn end_with(&mut self, end_cycles: Cycles, descendant_count: usize) {
        self.end_cycles = end_cycles;
        self._descendant_count = descendant_count;
    }

    pub(crate) fn is_root(&self) -> bool {
        self.parent_id == SpanId::new(0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ExternalSpan {
    id: SpanId,
    parent_id: SpanId,
    begin_cycles: Cycles,
    event: &'static str,
}

impl ExternalSpan {
    pub fn new(id: SpanId, parent_id: SpanId, begin_cycles: Cycles, event: &'static str) -> Self {
        ExternalSpan {
            id,
            parent_id,
            begin_cycles,
            event,
        }
    }

    pub fn to_span(&self, end_cycles: Cycles) -> Span {
        Span {
            id: self.id,
            parent_id: self.parent_id,
            begin_cycles: self.begin_cycles,
            event: self.event,
            end_cycles,
            _descendant_count: 0,
        }
    }

    pub fn id(&self) -> SpanId {
        self.id
    }
}
