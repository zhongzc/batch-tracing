use crate::local::span_line::SPAN_LINE;
use crate::span::cycle::{Cycle, Realtime};

pub fn cycle_to_realtime(cycle: Cycle) -> Realtime {
    SPAN_LINE.with(|span_line| {
        let span_line = unsafe { &*span_line.get() };
        span_line.cycle_to_realtime(cycle)
    })
}
