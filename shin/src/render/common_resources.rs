use crate::render::pipelines::{CommonBinds, Pipelines};
use crate::render::{
    BindGroupLayouts, PosColTexVertex, PosVertex, SubmittingEncoder, TextureBindGroup, VertexSource,
};
use cgmath::{Matrix4, Vector4};

pub struct GpuCommonResources {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub texture_format: wgpu::TextureFormat,
    pub pipelines: Pipelines,
    pub common_binds: CommonBinds,
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
            .draw(&self.common_binds, render_pass, source, texture, transform);
    }

    pub fn draw_fill<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, PosVertex>,
        color: Vector4<f32>,
    ) {
        self.pipelines
            .fill
            .draw(&self.common_binds, render_pass, source, color);
    }
}
