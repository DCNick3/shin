mod bind_groups;
mod camera;
mod common_resources;
pub mod dynamic_atlas;
pub mod overlay;
mod pillarbox;
mod pipelines;
mod render_target;
mod vertex_buffer;

pub use bind_groups::{BindGroupLayouts, TextureBindGroup};
pub use camera::{Camera, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
pub use common_resources::GpuCommonResources;
pub use pillarbox::Pillarbox;
pub use pipelines::{Pipelines, PosColTexVertex, PosVertex, TextVertex, VertexSource};
pub use render_target::RenderTarget;
pub use vertex_buffer::{IndexBuffer, PosVertexBuffer, SpriteVertexBuffer, Vertex, VertexBuffer};

use enum_dispatch::enum_dispatch;
use glam::Mat4;
use std::ops::{Deref, DerefMut};

use crate::layer::UserLayer;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub struct SubmittingEncoder<'q> {
    encoder: Option<wgpu::CommandEncoder>,
    queue: &'q wgpu::Queue,
}

impl<'q> Drop for SubmittingEncoder<'q> {
    fn drop(&mut self) {
        self.queue
            .submit(Some(self.encoder.take().unwrap().finish()));
    }
}

impl<'q> Deref for SubmittingEncoder<'q> {
    type Target = wgpu::CommandEncoder;

    fn deref(&self) -> &Self::Target {
        self.encoder.as_ref().unwrap()
    }
}

impl<'q> DerefMut for SubmittingEncoder<'q> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.encoder.as_mut().unwrap()
    }
}

#[enum_dispatch]
pub trait Renderable {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Mat4,
    );
    fn resize(&mut self, resources: &GpuCommonResources);
}
