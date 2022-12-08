mod bind_groups;
mod camera;
mod common_resources;
mod pillarbox;
mod pipelines;
mod render_target;
mod vertex_buffer;
mod window;

pub use bind_groups::{BindGroupLayouts, CameraBindGroup, TextureBindGroup};
pub use camera::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
pub use common_resources::GpuCommonResources;
pub use pipelines::{PosColTexVertex, PosVertex, VertexSource};
pub use render_target::RenderTarget;
pub use vertex_buffer::{IndexBuffer, SpriteVertexBuffer, Vertex, VertexBuffer};

use enum_dispatch::enum_dispatch;
use std::ops::{Deref, DerefMut};

use crate::layer::UserLayer;

pub use window::run;

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
    );
    fn resize(&mut self, resources: &GpuCommonResources, size: (u32, u32));
}
