use crate::local::acquirer_group::{registered_acquirer_group, root_acquirer_group};
use crate::local::scope_guard::LocalScopeGuard;
use crate::span::Span;
use crate::trace::acquirer::{Acquirer, AcquirerGroup};
use crossbeam_channel::Sender;
use std::sync::Arc;

pub struct Scope {
    acquirer_group: Option<Arc<AcquirerGroup>>,
}

impl Scope {
    pub fn start_scope(&self) -> LocalScopeGuard {
        LocalScopeGuard::new(self.acquirer_group.as_ref().cloned())
    }
}

impl Scope {
    pub(crate) fn new_root_scope(event: &'static str, sender: Arc<Sender<Vec<Span>>>) -> Self {
        Self {
            acquirer_group: Some(Arc::new(root_acquirer_group(Acquirer::new(sender), event))),
        }
    }

    pub(crate) fn new_scope(event: &'static str) -> Self {
        Self {
            acquirer_group: registered_acquirer_group(event).map(Arc::new),
        }
    }
}
