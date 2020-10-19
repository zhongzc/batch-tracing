use std::sync::atomic::{AtomicU32, Ordering};

/// TODO: doc
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SpanId(pub u64);

impl SpanId {
    pub fn new(id: u64) -> Self {
        SpanId(id)
    }
}

pub trait IdGenerator {
    fn next_id(&self) -> SpanId;
}

pub struct TempIdGenerator;

static NEXT_ID: AtomicU32 = AtomicU32::new(100);

impl IdGenerator for TempIdGenerator {
    fn next_id(&self) -> SpanId {
        SpanId::new(NEXT_ID.fetch_add(1, Ordering::SeqCst) as _)
    }
}
