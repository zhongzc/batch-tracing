use crate::local::registry::Listener;
use crate::local::span_line::SPAN_LINE;
use crate::span::Span;
use crate::trace::acquirer::AcquirerGroup;
use crate::trace::tracer::Tracer;
use std::sync::Arc;

enum State {
    Registered(Listener),
    Unregistered(Arc<AcquirerGroup>),
}

pub struct Trapper {
    caught_spans: Vec<Span>,

    /// If trapper is registered, a listener is stored here,
    /// or, a acquirer group to register is stored.
    /// Using `Option` is for switching state.
    state: Option<State>,
}

impl Trapper {
    pub fn new(acquirer_group: Option<Arc<AcquirerGroup>>) -> Self {
        Self {
            caught_spans: vec![],
            state: acquirer_group.map(|acg| State::Unregistered(acg)),
        }
    }

    pub fn set_trap(&mut self) {
        let state = if let Some(s) = self.state.take() {
            s
        } else {
            return;
        };

        match state {
            r @ State::Registered(_) => self.state = Some(r),
            State::Unregistered(acg) => SPAN_LINE.with(|span_line| {
                let span_line = unsafe { &mut *span_line.get() };
                let listener = span_line.register_now(acg);
                self.state = Some(State::Registered(listener))
            }),
        }
    }

    pub fn pull(&mut self) {
        let state = if let Some(s) = self.state.take() {
            s
        } else {
            return;
        };

        match state {
            u @ State::Unregistered(_) => self.state = Some(u),
            State::Registered(listener) => SPAN_LINE.with(|span_line| {
                let span_line = unsafe { &mut *span_line.get() };
                self.caught_spans.extend(span_line.spans_from(listener));
                let acg = span_line.unregister(listener);
                self.state = Some(State::Unregistered(acg));
            }),
        }
    }
}

impl Trapper {
    /// Only called by `drop`
    fn submit(&mut self) {
        if let Some(State::Unregistered(acg)) = self.state.take() {
            if !self.caught_spans.is_empty() {
                acg.submit(std::mem::take(&mut self.caught_spans));
            }
        }
    }
}

impl Drop for Trapper {
    fn drop(&mut self) {
        self.pull();
        self.submit()
    }
}

impl !Send for Trapper {}
impl !Sync for Trapper {}
