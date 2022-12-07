use derive_more::{Add, AddAssign, Sub, SubAssign};
use std::ops::Div;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Add, AddAssign, Sub, SubAssign, PartialEq, PartialOrd)]
pub struct Ticks(pub f32);

pub const TICKS_PER_SECOND: f32 = 60.0;

impl Ticks {
    pub const ZERO: Self = Self(0.0);

    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds * TICKS_PER_SECOND)
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self::from_seconds(duration.as_secs_f32())
    }

    pub fn as_seconds(&self) -> f32 {
        self.0 / TICKS_PER_SECOND
    }

    pub fn as_duration(&self) -> Duration {
        Duration::from_secs_f32(self.as_seconds())
    }
}

impl Div for Ticks {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

pub struct UpdateContext<'a> {
    time: &'a bevy_time::Time,
}

impl<'a> UpdateContext<'a> {
    pub fn new(time: &'a bevy_time::Time) -> Self {
        Self { time }
    }

    pub fn delta(&self) -> Duration {
        self.time.delta()
    }
    pub fn delta_ticks(&self) -> Ticks {
        Ticks::from_seconds(self.time.delta_seconds())
    }
}

pub trait Updatable {
    fn update(&mut self, context: &UpdateContext);
}
