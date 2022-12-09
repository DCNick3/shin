use crate::game_data::GameData;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use enum_dispatch::enum_dispatch;
use std::ops::Div;
use std::time::Duration;

use crate::layer::UserLayer;
use crate::render::GpuCommonResources;

// TODO: move to shin_core
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
    pub time: &'a bevy_time::Time,
    pub gpu_resources: &'a GpuCommonResources,
    pub game_data: &'a GameData,
}

impl<'a> UpdateContext<'a> {
    pub fn time_delta(&self) -> Duration {
        self.time.delta()
    }
    pub fn time_delta_ticks(&self) -> Ticks {
        Ticks::from_seconds(self.time.delta_seconds())
    }
}

#[enum_dispatch]
pub trait Updatable {
    fn update(&mut self, context: &UpdateContext);
}
