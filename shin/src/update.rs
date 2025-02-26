use std::sync::Arc;

use shin_core::{primitives::update::FrameId, time::Ticks};

use crate::{asset::system::AssetServer, render::PreRenderContext};

pub struct UpdateContext<'immutable, 'pre_render, 'pipelines, 'dynbuffer, 'encoder> {
    pub frame_id: FrameId,
    pub delta_ticks: Ticks,
    pub asset_server: &'immutable Arc<AssetServer>,
    pub pre_render: &'pre_render mut PreRenderContext<'immutable, 'pipelines, 'dynbuffer, 'encoder>,
}

pub struct AdvUpdateContext<'a> {
    pub frame_id: FrameId,
    pub delta_ticks: Ticks,
    #[expect(unused)] // for future stuff
    pub asset_server: &'a Arc<AssetServer>,
    // we do not provide access to pre-render context here because there is another method for it (at least on layers)
    // you can create render resources and schedule transmissions though
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,

    pub are_animations_allowed: bool,
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
