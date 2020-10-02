use crate::local::span_line::SPAN_LINE;
use crate::span::span_queue::Finisher;

pub struct LocalSpanGuard {
    finisher: Option<Finisher>,
}

impl LocalSpanGuard {
    pub fn new(event: &'static str) -> Self {
        SPAN_LINE.with(|span_line| {
            let span_line = unsafe { &mut *span_line.get() };

            let finisher = span_line.start_span(event);
            Self { finisher }
        })
    }
}

impl Drop for LocalSpanGuard {
    fn drop(&mut self) {
        if let Some(finisher) = self.finisher.take() {
            SPAN_LINE.with(|span_line| {
                let span_line = unsafe { &mut *span_line.get() };

                span_line.finish_span(finisher);
            })
        }
    }
}

impl !Send for LocalSpanGuard {}
impl !Sync for LocalSpanGuard {}
