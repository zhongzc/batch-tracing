use crate::local::span_line::SPAN_LINE;
use crate::span::cycle::{Cycle, Realtime};

pub fn cycle_to_realtime(cycle: Cycle) -> Realtime {
    SPAN_LINE.with(|span_line| {
        let span_line = span_line.borrow();
        span_line.cycle_to_realtime(cycle)
    })
}
