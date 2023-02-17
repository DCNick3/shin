use crate::vertices::{PosVertex, VertexSource};
use crate::{pipelines, BindGroupLayouts};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use std::mem;
use wgpu::include_wgsl;

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct FillParams {
    pub transform: Mat4,
    pub color: Vec4,
}

pub struct FillPipeline(wgpu::RenderPipeline);

impl FillPipeline {
    pub fn new(
        device: &wgpu::Device,
        _bind_group_layouts: &BindGroupLayouts,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("fill.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fill_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..mem::size_of::<FillParams>() as u32,
            }],
        });

        Self(pipelines::make_pipeline(
            device,
            texture_format,
            shader_module,
            layout,
            PosVertex::desc(),
            Some(wgpu::BlendState::ALPHA_BLENDING),
            "fill_pipeline",
        ))
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, PosVertex>,
        transform: Mat4,
        color: Vec4,
    ) {
        render_pass.set_pipeline(&self.0);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[FillParams { transform, color }]),
        );
        source.draw(render_pass);
    }
}
