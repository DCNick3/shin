use crate::render::bind_group_layouts::BindGroupLayouts;
use crate::render::RenderContext;
use cgmath::{Vector2, Vector3, Vector4};
use std::ops::Range;

#[repr(C)]
#[derive(Copy, Clone, Debug, wrld::Desc, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    #[f32x3(0)]
    pub position: Vector3<f32>,
    #[f32x4(1)]
    pub color: Vector4<f32>,
    #[f32x2(2)]
    pub texture_coordinate: Vector2<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, wrld::Desc, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PositionVertex {
    #[f32x3(0)]
    pub position: Vector3<f32>,
}

pub struct CommonBinds<'a> {
    pub camera: &'a wgpu::BindGroup,
}

pub enum DrawSource<'a, T> {
    VertexBuffer {
        vertex_buffer: &'a wgpu::Buffer, // TODO: support multiple vertex buffers
        vertices: Range<u32>,
        instances: Range<u32>,
        phantom: std::marker::PhantomData<T>,
    },
    VertexIndexBuffer {
        vertex_buffer: &'a wgpu::Buffer,
        index_buffer: &'a wgpu::Buffer,
        indices: Range<u32>,
        instances: Range<u32>,
    },
}

impl<'a, T> DrawSource<'a, T> {
    pub fn draw(&self, render_context: &mut RenderContext<'a, '_>) {
        match self {
            DrawSource::VertexBuffer {
                vertex_buffer,
                vertices,
                instances,
                phantom: _,
            } => {
                render_context
                    .render_pass
                    .set_vertex_buffer(0, vertex_buffer.slice(..));
                render_context
                    .render_pass
                    .draw(vertices.clone(), instances.clone());
            }
            DrawSource::VertexIndexBuffer {
                vertex_buffer,
                index_buffer,
                indices,
                instances,
            } => {
                render_context
                    .render_pass
                    .set_vertex_buffer(0, vertex_buffer.slice(..));
                render_context
                    .render_pass
                    .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_context
                    .render_pass
                    .draw_indexed(indices.clone(), 0, instances.clone());
            }
        }
    }
}

pub mod sprite {
    use super::SpriteVertex;
    use crate::asset::picture::GpuPicture;
    use crate::render::bind_group_layouts::BindGroupLayouts;
    use crate::render::pipelines::DrawSource;
    use wgpu::include_wgsl;

    pub fn make_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bind_group_layouts: &BindGroupLayouts,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(include_wgsl!("sprite.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layouts.camera, &bind_group_layouts.picture],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[SpriteVertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    pub fn draw<'a>(
        ctx: &mut crate::render::RenderContext<'a, '_>,
        source: DrawSource<'a, SpriteVertex>,
        picture: &'a GpuPicture,
    ) {
        ctx.render_pass.set_pipeline(&ctx.pipelines.sprite);
        ctx.render_pass
            .set_bind_group(0, &ctx.common_binds.camera, &[]);
        // TODO: use origin info from the picture
        ctx.render_pass.set_bind_group(1, &picture.bind_group, &[]);
        source.draw(ctx);
    }
}

pub mod fill {
    use super::PositionVertex;
    use crate::render::bind_group_layouts::BindGroupLayouts;
    use crate::render::pipelines::DrawSource;
    use cgmath::Vector4;
    use std::mem;
    use wgpu::include_wgsl;

    pub fn make_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bind_group_layouts: &BindGroupLayouts,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(include_wgsl!("fill.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fill_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layouts.camera],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..mem::size_of::<Vector4<f32>>() as u32,
            }],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fill_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[PositionVertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    pub fn draw<'a>(
        ctx: &mut crate::render::RenderContext<'a, '_>,
        source: DrawSource<'a, PositionVertex>,
        color: Vector4<f32>,
    ) {
        ctx.render_pass.set_pipeline(&ctx.pipelines.fill);
        ctx.render_pass
            .set_bind_group(0, &ctx.common_binds.camera, &[]);
        ctx.render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::cast_slice(&[color]),
        );
        source.draw(ctx);
    }
}

pub struct Pipelines {
    pub sprite: wgpu::RenderPipeline,
    pub fill: wgpu::RenderPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        surface_format: wgpu::TextureFormat,
    ) -> Pipelines {
        Pipelines {
            sprite: sprite::make_pipeline(device, surface_format, bind_group_layouts),
            fill: fill::make_pipeline(device, surface_format, bind_group_layouts),
        }
    }
}
