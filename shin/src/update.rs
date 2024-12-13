use std::sync::Arc;

use shin_core::time::Ticks;

use crate::{asset::system::AssetServer, layer::PreRenderContext};

pub struct UpdateContext<'immutable, 'pre_render, 'pipelines, 'dynbuffer, 'encoder> {
    pub delta_time: Ticks,
    pub asset_server: &'immutable Arc<AssetServer>,
    pub pre_render: &'pre_render mut PreRenderContext<'immutable, 'pipelines, 'dynbuffer, 'encoder>,
}

pub struct AdvUpdateContext<'a> {
    pub delta_time: Ticks,
    #[expect(unused)] // for future stuff
    pub asset_server: &'a Arc<AssetServer>,
    pub are_animations_allowed: bool,
}

impl<'a> AdvUpdateContext<'a> {
    #[expect(unused)] // for future stuff
    pub fn from_update_context(
        context: &'a UpdateContext<'a, '_, '_, '_, '_>,
        are_animations_allowed: bool,
    ) -> Self {
        Self {
            delta_time: context.delta_time,
            asset_server: context.asset_server,
            are_animations_allowed,
        }
    }
}

pub trait Updatable {
    fn update(&mut self, context: &mut UpdateContext);
}

impl<T: Updatable> Updatable for Box<T> {
    #[inline]
    fn update(&mut self, context: &mut UpdateContext) {
        (**self).update(context)
    }
}

pub trait AdvUpdatable {
    fn update(&mut self, context: &AdvUpdateContext);
}

impl<T: AdvUpdatable> AdvUpdatable for Box<T> {
    #[inline]
    fn update(&mut self, context: &AdvUpdateContext) {
        (**self).update(context)
    }
}
