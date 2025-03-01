use glam::Mat4;
use shin_render::{
    dynamic_buffer::DynamicBuffer,
    pipelines::PipelineStorage,
    render_pass::RenderPass,
    render_texture::RenderTexture,
    resize::SurfaceResizeSource,
    shaders::types::{
        RenderCloneCtx,
        texture::{DepthStencilTarget, TextureSamplerStore, TextureTarget},
    },
    shin_orthographic_projection_matrix,
};
use winit::dpi::PhysicalSize;

#[expect(unused)]
pub mod overlay;
pub mod render_texture_holder;

pub const VIRTUAL_CANVAS_SIZE: PhysicalSize<u32> = PhysicalSize::new(1920, 1080);
pub const VIRTUAL_CANVAS_SIZE_VEC: glam::Vec2 = glam::vec2(
    VIRTUAL_CANVAS_SIZE.width as f32,
    VIRTUAL_CANVAS_SIZE.height as f32,
);

pub fn centered_projection_matrix() -> Mat4 {
    shin_orthographic_projection_matrix(
        -VIRTUAL_CANVAS_SIZE_VEC.x / 2.0,
        VIRTUAL_CANVAS_SIZE_VEC.x / 2.0,
        VIRTUAL_CANVAS_SIZE_VEC.y / 2.0,
        -VIRTUAL_CANVAS_SIZE_VEC.y / 2.0,
        -1.0,
        1.0,
    )
}

pub fn top_left_projection_matrix() -> Mat4 {
    shin_orthographic_projection_matrix(
        0.0,
        VIRTUAL_CANVAS_SIZE_VEC.x,
        VIRTUAL_CANVAS_SIZE_VEC.y,
        0.0,
        -1.0,
        1.0,
    )
}

pub fn normalized_projection_matrix() -> Mat4 {
    shin_orthographic_projection_matrix(0.0, 1.0, 1.0, 0.0, -1.0, 1.0)
}

pub struct PreRenderContext<'immutable, 'pipelines, 'dynbuffer, 'encoder> {
    pub device: &'immutable wgpu::Device,
    pub queue: &'immutable wgpu::Queue,
    pub resize_source: &'immutable SurfaceResizeSource,
    pub sampler_store: &'immutable TextureSamplerStore,
    pub depth_stencil: DepthStencilTarget<'immutable>,

    pub pipeline_storage: &'pipelines mut PipelineStorage,
    pub dynamic_buffer: &'dynbuffer mut DynamicBuffer,
    pub encoder: &'encoder mut wgpu::CommandEncoder,
}

impl PreRenderContext<'_, '_, '_, '_> {
    pub fn new_render_texture(&self, label: String) -> RenderTexture {
        RenderTexture::new(self.device.clone(), self.resize_source.handle(), label)
    }

    pub fn begin_pass(
        &mut self,
        target: TextureTarget,
        depth_stencil: Option<DepthStencilTarget>,
        label: &str,
    ) -> RenderPass {
        RenderPass::new(
            self.pipeline_storage,
            self.dynamic_buffer,
            self.sampler_store,
            self.device,
            self.encoder,
            target,
            depth_stencil,
            None,
            label,
        )
    }

    pub fn render_clone_ctx(&mut self) -> RenderCloneCtx {
        RenderCloneCtx {
            device: self.device,
            encoder: self.encoder,
        }
    }
}
