use crate::local::span_line::SPAN_LINE;
use crate::span::span_queue::Finisher;

pub struct LocalSpanGuard {
    finisher: Option<Finisher>,
}

impl LocalSpanGuard {
    #[inline]
    pub(crate) fn new(event: &'static str) -> Self {
        let span_line = SPAN_LINE.with(|span_line| unsafe { &mut *span_line.get() });
        let finisher = span_line.start_span(event);
        Self { finisher }
    }
}

impl Drop for LocalSpanGuard {
    #[inline]
    fn drop(&mut self) {
        if let Some(finisher) = self.finisher.take() {
            let span_line = SPAN_LINE.with(|span_line| unsafe { &mut *span_line.get() });
            span_line.finish_span(finisher);
        }
    }
}

impl !Send for LocalSpanGuard {}

impl !Sync for LocalSpanGuard {}
