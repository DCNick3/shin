use crate::asset::AnyAssetServer;
use enum_dispatch::enum_dispatch;
use shin_core::vm::command::time::Ticks;
use std::sync::Arc;
use std::time::Duration;

use crate::layer::UserLayer;
use crate::render::GpuCommonResources;

pub struct UpdateContext<'a> {
    pub time: &'a bevy_time::Time,
    pub gpu_resources: &'a Arc<GpuCommonResources>,
    pub asset_server: &'a Arc<AnyAssetServer>,
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
