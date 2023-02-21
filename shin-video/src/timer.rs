use shin_core::time::Ticks;

pub struct IndependentTimer {
    /// How many time units are there in one second
    time_base: u32,
    /// How many time units have passed since the start of the timer
    time: u64,
}

impl IndependentTimer {
    pub fn new(time_base: u32) -> IndependentTimer {
        IndependentTimer { time_base, time: 0 }
    }

    pub fn update(&mut self, delta_time: Ticks) {
        self.time += (delta_time.as_seconds() as f64 * self.time_base as f64) as u64;
    }

    pub fn time(&self) -> u64 {
        self.time
    }
}

pub enum Timer {
    Independent(IndependentTimer),
}

impl Timer {
    pub fn new_independent(time_base: u32) -> Timer {
        Timer::Independent(IndependentTimer::new(time_base))
    }

    pub fn update(&mut self, delta_time: Ticks) {
        match self {
            Timer::Independent(timer) => timer.update(delta_time),
        }
    }

    pub fn time(&self) -> u64 {
        match self {
            Timer::Independent(timer) => timer.time(),
        }
    }
}
