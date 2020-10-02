use crate::local::acquirer_group::{registered_acquirer_group, root_acquirer_group};
use crate::local::trapper::Trapper;
use crate::trace::acquirer::{Acquirer, AcquirerGroup};
use std::sync::Arc;

pub struct Tracer {
    acquirer_group: Option<Arc<AcquirerGroup>>,
}

impl Tracer {
    pub fn root(event: &'static str) -> Self {
        Self {
            acquirer_group: Some(Arc::new(root_acquirer_group(Acquirer::new(), event))),
        }
    }

    pub fn spawn(event: &'static str) -> Self {
        Self {
            acquirer_group: registered_acquirer_group(event).map(|acg| Arc::new(acg)),
        }
    }

    pub fn new_trapper(&self) -> Trapper {
        Trapper::new(self.acquirer_group.as_ref().cloned())
    }
}
