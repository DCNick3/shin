use crate::render::camera::Camera;
use crate::render::pipelines::Pipelines;
use crate::render::{
    BindGroupLayouts, PosColTexVertex, PosVertex, SubmittingEncoder, TextureBindGroup, VertexSource,
};
use cgmath::{Matrix4, Vector4};

pub struct GpuCommonResources {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub camera: Camera,
    pub pipelines: Pipelines,
    pub bind_group_layouts: BindGroupLayouts,
}

impl GpuCommonResources {
    pub fn start_encoder(&self) -> SubmittingEncoder {
        SubmittingEncoder {
            encoder: Some(
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("encoder"),
                    }),
            ),
            queue: &self.queue,
        }
    }

    pub fn draw_sprite<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, PosColTexVertex>,
        texture: &'a TextureBindGroup,
        transform: Matrix4<f32>,
    ) {
        self.pipelines
            .sprite
            .draw(render_pass, source, texture, transform);
    }

    pub fn draw_fill<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, PosVertex>,
        transform: Matrix4<f32>,
        color: Vector4<f32>,
    ) {
        self.pipelines
            .fill
            .draw(render_pass, source, transform, color);
    }

    pub fn current_render_buffer_size(&self) -> (u32, u32) {
        self.camera.render_buffer_size()
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.camera.projection_matrix()
    }
}
