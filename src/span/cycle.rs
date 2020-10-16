/// TODO: doc
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Cycles(u64);

impl Cycles {
    pub fn new(cycles: u64) -> Self {
        Cycles(cycles)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

pub trait Clock {
    fn now(&self) -> Cycles;
}

pub struct TempClock;

impl Clock for TempClock {
    fn now(&self) -> Cycles {
        Cycles::new(minstant::now())
    }
}
