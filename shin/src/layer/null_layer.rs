use crate::layer::{Layer, LayerProperties};
use crate::update::{Updatable, UpdateContext};
use glam::Mat4;
use shin_render::GpuCommonResources;
use shin_render::Renderable;
use std::fmt::Debug;

pub struct NullLayer {
    props: LayerProperties,
}

impl NullLayer {
    pub fn new() -> Self {
        Self {
            props: LayerProperties::new(),
        }
    }
}

impl Renderable for NullLayer {
    fn render<'enc>(
        &'enc self,
        _resources: &'enc GpuCommonResources,
        _render_pass: &mut wgpu::RenderPass<'enc>,
        _transform: Mat4,
        _projection: Mat4,
    ) {
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for NullLayer {
    fn update(&mut self, _ctx: &UpdateContext) {}
}

impl Debug for NullLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NullLayer").finish()
    }
}

impl Layer for NullLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
