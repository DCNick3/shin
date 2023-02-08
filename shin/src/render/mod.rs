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

/// A trait for elements that can be rendered
///
/// Most elements will be containers, containing other elements to render.
#[enum_dispatch]
pub trait Renderable {
    /// Renders an element on the screen
    ///
    /// # Arguments
    ///
    /// * `resources` - The common resources used by the renderer
    /// * `render_pass` - The render pass to encode commands to
    /// * `transform` - The transform matrix to apply to the element
    /// * `projection` - The projection matrix to apply to the element
    ///
    /// # Notes
    ///
    /// The `projection` matrix is used to convert from virtual screen space to real screen space.
    /// The `transform` matrix represents inherited transformations from parent elements.
    ///
    /// This distinction is needed to allow for rendering using intermediate render targets.
    /// In this case the `transform` matrix is preserved and passed into inner elements.
    /// The `projection` matrix is used only to render the intermediate render target to the screen.
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Mat4,
        projection: Mat4,
    );

    /// Notifies of window resize
    ///
    /// If a renderable element has an intermediate render target, it should resize it here.
    fn resize(&mut self, resources: &GpuCommonResources);
}
