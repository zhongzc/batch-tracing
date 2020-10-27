use crate::local::registry::Listener;
use crate::local::span_line::SPAN_LINE;
use crate::span::Span;
use crate::trace::acquirer::AcquirerGroup;
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
            state: acquirer_group.map(State::Unregistered),
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
                let mut span_line = span_line.borrow_mut();
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
                let mut span_line = span_line.borrow_mut();
                let (acg, spans) = span_line.unregister_and_collect(listener);
                self.caught_spans.extend(spans);
                self.state = Some(State::Unregistered(acg));
            }),
        }
    }

    pub fn submit(&mut self) {
        if let Some(State::Unregistered(acg)) = self.state.take() {
            if !self.caught_spans.is_empty() {
                acg.submit(std::mem::take(&mut self.caught_spans));
            }
        }
    }
}

impl !Send for Trapper {}
impl !Sync for Trapper {}
