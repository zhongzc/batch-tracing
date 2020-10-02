use crate::local::span_line::SPAN_LINE;
use crate::span::ExternalSpan;
use crate::trace::acquirer::{Acquirer, AcquirerGroup};

/// Return registered acquirers from the current thread, or `None` if
/// there're no acquires registered.
pub fn registered_acquirer_group(event: &'static str) -> Option<AcquirerGroup> {
    SPAN_LINE.with(|span_line| {
        let span_line = unsafe { &mut *span_line.get() };
        span_line.registered_acquirer_group(event)
    })
}

pub fn root_acquirer_group(acquirer: Acquirer, event: &'static str) -> AcquirerGroup {
    SPAN_LINE.with(|span_line| {
        let span_line = unsafe { &mut *span_line.get() };
        AcquirerGroup::new(span_line.start_root_external_span(event), vec![acquirer])
    })
}

pub fn submit_task_span(acg: &AcquirerGroup, task_span: &ExternalSpan) {
    SPAN_LINE.with(|span_line| {
        let span_line = unsafe { &*span_line.get() };
        let s = span_line.finish_external_span(task_span);
        acg.submit_task_span(s);
    });
}
