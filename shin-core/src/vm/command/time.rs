use derive_more::{Add, AddAssign, Sub, SubAssign};
use float_ord::FloatOrd;
use std::ops::Div;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Add, AddAssign, Sub, SubAssign)]
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

impl PartialEq for Ticks {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.0).eq(&FloatOrd(other.0))
    }
}

impl Eq for Ticks {}

impl PartialOrd for Ticks {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        FloatOrd(self.0).partial_cmp(&FloatOrd(other.0))
    }
}

impl Ord for Ticks {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.0).cmp(&FloatOrd(other.0))
    }
}
