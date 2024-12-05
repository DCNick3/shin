use std::{sync::Arc, time::Duration};

use enum_dispatch::enum_dispatch;
use shin_core::time::Ticks;

use crate::{asset::system::AssetServer, layer::user::UserLayer, time::Time};

pub struct UpdateContext<'a> {
    pub delta_time: Ticks,
    // pub time: &'a Time,
    // pub gpu_resources: &'a Arc<GpuCommonResources>,
    pub asset_server: &'a Arc<AssetServer>,
    // pub raw_input_state: &'a RawInputState,
}

impl<'a> UpdateContext<'a> {
    // pub fn time_delta(&self) -> Duration {
    //     self.time.delta()
    // }
    // pub fn time_delta_ticks(&self) -> Ticks {
    //     Ticks::from_seconds(self.time.delta_seconds())
    // }
}

pub trait Updatable {
    fn update(&mut self, context: &UpdateContext);
}
