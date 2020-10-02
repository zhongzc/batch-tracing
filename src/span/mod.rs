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

    // post process to fill
    end_cycles: Cycles,
    descendant_count: usize,
}

impl Span {
    pub fn begin_with(
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
            descendant_count: 0,
        }
    }

    pub fn end_with(&mut self, end_cycles: Cycles, descendant_count: usize) {
        self.end_cycles = end_cycles;
        self.descendant_count = descendant_count;
    }

    pub fn is_root(&self) -> bool {
        self.id == SpanId::new(0)
    }

    pub fn id(&self) -> SpanId {
        self.id
    }
    pub fn parent_id(&self) -> SpanId {
        self.parent_id
    }
    pub fn begin_cycles(&self) -> Cycles {
        self.begin_cycles
    }
    pub fn event(&self) -> &'static str {
        self.event
    }
    pub fn end_cycles(&self) -> Cycles {
        self.end_cycles
    }
    pub fn descendant_count(&self) -> usize {
        self.descendant_count
    }
    pub fn set_id(&mut self, id: SpanId) {
        self.id = id;
    }
    pub fn set_parent_id(&mut self, parent_id: SpanId) {
        self.parent_id = parent_id;
    }
    pub fn set_begin_cycles(&mut self, begin_cycles: Cycles) {
        self.begin_cycles = begin_cycles;
    }
    pub fn set_event(&mut self, event: &'static str) {
        self.event = event;
    }
    pub fn set_end_cycles(&mut self, end_cycles: Cycles) {
        self.end_cycles = end_cycles;
    }
    pub fn set_descendant_count(&mut self, descendant_count: usize) {
        self.descendant_count = descendant_count;
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
            descendant_count: 0,
        }
    }

    pub fn id(&self) -> SpanId {
        self.id
    }
}
