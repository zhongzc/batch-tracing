#![feature(map_first_last)]
#![feature(negative_impls)]

use std::sync::Arc;

pub use crate::local::span_guard::LocalSpanGuard;
pub use crate::span::Span;
pub use crate::trace::collector::Collector;
pub use crate::trace::scope::Scope;
use std::sync::atomic::AtomicBool;

pub mod collections;

pub(crate) mod local;
pub(crate) mod span;
pub(crate) mod trace;

pub fn root_scope(event: &'static str) -> (Scope, Collector) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let closed = Arc::new(AtomicBool::new(false));
    let scope = Scope::new_root_scope(event, tx, Arc::clone(&closed));
    let collector = Collector::new(rx, closed);
    (scope, collector)
}

pub fn spawn_scope(event: &'static str) -> Scope {
    Scope::new_scope(event)
}

pub fn new_span(event: &'static str) -> LocalSpanGuard {
    LocalSpanGuard::new(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_utils::sync::WaitGroup;

    fn four_spans() {
        {
            // wide
            for _ in 0..2 {
                let _g = new_span("iter span");
            }
        }

        {
            fn rec(mut i: u32) {
                i -= 1;

                let _g = new_span("rec span");

                if i > 0 {
                    rec(i);
                }
            }

            // deep
            rec(2);
        }
    }

    #[test]
    fn single_thread_single_scope() {
        let spans = {
            let (root_scope, collector) = root_scope("root");
            let _sg = root_scope.start_scope();

            four_spans();

            collector
        }
        .collect();

        assert_eq!(spans.len(), 5);
    }

    #[test]
    fn single_thread_multiple_scopes() {
        let (spans1, spans2, spans3) = {
            let (c1, c2, c3) = {
                let (root_scope1, collector1) = root_scope("root1");
                let (root_scope2, collector2) = root_scope("root2");
                let (root_scope3, collector3) = root_scope("root3");

                let _sg1 = root_scope1.start_scope();
                let _sg2 = root_scope2.start_scope();
                let _sg3 = root_scope3.start_scope();

                four_spans();

                (collector1, collector2, collector3)
            };

            (c1.collect(), c2.collect(), c3.collect())
        };

        assert_eq!(spans1.len(), 5);
        assert_eq!(spans2.len(), 5);
        assert_eq!(spans3.len(), 5);
    }

    #[test]
    fn multiple_threads_single_scope() {
        let spans = {
            let (scope, collector) = root_scope("root");

            let _sg = scope.start_scope();
            let wg = WaitGroup::new();

            for _ in 0..4 {
                let wg = wg.clone();
                let scope = spawn_scope("cross-thread");
                std::thread::spawn(move || {
                    let _sg = scope.start_scope();

                    four_spans();

                    drop(wg);
                });
            }

            four_spans();

            // wait for all threads to be done
            wg.wait();

            collector
        }
        .collect();

        assert_eq!(spans.len(), 29);
    }

    #[test]
    fn multiple_threads_multiple_scopes() {
        let (spans1, spans2) = {
            let (c1, c2) = {
                let (scope1, collector1) = root_scope("root1");
                let (scope2, collector2) = root_scope("root2");

                let _sg1 = scope1.start_scope();
                let _sg2 = scope2.start_scope();
                let wg = WaitGroup::new();

                for _ in 0..4 {
                    let wg = wg.clone();
                    let scope = spawn_scope("cross-thread");
                    std::thread::spawn(move || {
                        let _sg = scope.start_scope();

                        four_spans();

                        drop(wg);
                    });
                }

                four_spans();

                // wait for all threads to be done
                wg.wait();

                (collector1, collector2)
            };

            (c1.collect(), c2.collect())
        };

        assert_eq!(spans1.len(), 29);
        assert_eq!(spans2.len(), 29);
    }
}
