pub mod bind_group_layouts;
mod camera;
mod picture_layer;
mod pillarbox;
mod pipelines;
mod window;

use crate::render::bind_group_layouts::BindGroupLayouts;
use crate::render::pipelines::{CommonBinds, Pipelines};
pub use window::run;

pub struct RenderContext<'cmd, 'pass> {
    pub device: &'cmd wgpu::Device,
    pub render_pass: &'pass mut wgpu::RenderPass<'cmd>,
    pub pipelines: &'cmd Pipelines,
    pub common_binds: &'cmd CommonBinds<'cmd>,
    pub bind_group_layouts: &'cmd BindGroupLayouts,
}
