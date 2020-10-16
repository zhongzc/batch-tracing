use crate::local::trapper::Trapper;
use crate::trace::acquirer::AcquirerGroup;
use std::sync::Arc;

pub struct LocalScopeGuard {
    trapper: Trapper,
}

impl !Sync for LocalScopeGuard {}
impl !Send for LocalScopeGuard {}

impl LocalScopeGuard {
    pub fn new(acquirer_group: Option<Arc<AcquirerGroup>>) -> Self {
        let mut trapper = Trapper::new(acquirer_group);
        trapper.set_trap();
        Self { trapper }
    }
}

impl Drop for LocalScopeGuard {
    fn drop(&mut self) {
        self.trapper.pull();
        self.trapper.submit();
    }
}
