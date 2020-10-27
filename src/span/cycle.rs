use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

/// TODO: doc
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Cycle(pub u64);

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Realtime {
    pub ns: u64,
}

impl Cycle {
    pub fn new(cycles: u64) -> Self {
        Cycle(cycles)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

pub trait Clock {
    fn now(&self) -> Cycle;
    fn cycle_to_realtime(&self, cycle: Cycle) -> Realtime;
}

pub struct DefaultClock;

#[derive(Copy, Clone, Default)]
struct Anchor {
    pub realtime: Realtime,
    pub cycle: Cycle,
    pub cycles_per_second: u64,
}

impl Clock for DefaultClock {
    fn now(&self) -> Cycle {
        Cycle::new(minstant::now())
    }

    fn cycle_to_realtime(&self, cycle: Cycle) -> Realtime {
        let anchor = Self::anchor();
        if cycle > anchor.cycle {
            let forward_ns = ((cycle.0 - anchor.cycle.0) as u128 * 1_000_000_000
                / anchor.cycles_per_second as u128) as u64;
            Realtime {
                ns: anchor.realtime.ns + forward_ns,
            }
        } else {
            let backward_ns = ((anchor.cycle.0 - cycle.0) as u128 * 1_000_000_000
                / anchor.cycles_per_second as u128) as u64;
            Realtime {
                ns: anchor.realtime.ns - backward_ns,
            }
        }
    }
}

impl DefaultClock {
    fn anchor() -> Anchor {
        static mut ANCHOR: Anchor = Anchor {
            realtime: Realtime { ns: 0 },
            cycle: Cycle(0),
            cycles_per_second: 0,
        };
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let cycle = Cycle::new(minstant::now());
            let realtime = Realtime {
                ns: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("unexpected time drift")
                    .as_nanos() as u64,
            };

            unsafe {
                ANCHOR = Anchor {
                    realtime,
                    cycle,
                    cycles_per_second: minstant::cycles_per_second(),
                };
            }
        });

        unsafe { ANCHOR }
    }
}
