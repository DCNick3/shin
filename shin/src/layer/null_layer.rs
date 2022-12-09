use crate::layer::{Layer, LayerProperties};
use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use cgmath::Matrix4;

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
        _transform: Matrix4<f32>,
    ) {
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for NullLayer {
    fn update(&mut self, _ctx: &UpdateContext) {}
}

impl Layer for NullLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
