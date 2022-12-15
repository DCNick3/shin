use crate::render;
use crate::render::bind_groups::{BindGroupLayouts, TextureBindGroup};
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use std::mem;
use std::ops::Range;
use wgpu::include_wgsl;

#[repr(C)]
#[derive(Copy, Clone, Debug, wrld::Desc, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosColTexVertex {
    #[f32x3(0)]
    pub position: Vector3<f32>,
    #[f32x4(1)]
    pub color: Vector4<f32>,
    #[f32x2(2)]
    pub texture_coordinate: Vector2<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, wrld::Desc, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PosVertex {
    #[f32x3(0)]
    pub position: Vector3<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, wrld::Desc, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    #[f32x2(0)]
    pub position: Vector2<f32>,
    #[f32x2(1)]
    pub tex_position: Vector2<f32>,
    #[f32x3(2)]
    pub color: Vector3<f32>,
    #[f32(3)]
    pub time: f32,
    #[f32(4)]
    pub fade: f32,
}

pub enum VertexSource<'a, T> {
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

impl<'a, T> VertexSource<'a, T> {
    pub fn vertex_count(&self) -> u32 {
        match self {
            VertexSource::VertexBuffer { vertices, .. } => vertices.end - vertices.start,
            VertexSource::VertexIndexBuffer { indices, .. } => indices.end - indices.start,
        }
    }

    pub fn vertex_buffer(&self) -> &'a wgpu::Buffer {
        match self {
            VertexSource::VertexBuffer { vertex_buffer, .. } => vertex_buffer,
            VertexSource::VertexIndexBuffer { vertex_buffer, .. } => vertex_buffer,
        }
    }

    pub fn instances(&self) -> Range<u32> {
        match self {
            VertexSource::VertexBuffer { instances, .. } => instances.clone(),
            VertexSource::VertexIndexBuffer { instances, .. } => instances.clone(),
        }
    }

    pub fn with_index_buffer(self, index_buffer: &'a wgpu::Buffer, indices: Range<u32>) -> Self {
        VertexSource::VertexIndexBuffer {
            vertex_buffer: self.vertex_buffer(),
            index_buffer,
            indices,
            instances: self.instances(),
        }
    }
}

impl<'a, T> VertexSource<'a, T> {
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'a>) {
        match self {
            VertexSource::VertexBuffer {
                vertex_buffer,
                vertices,
                instances,
                phantom: _,
            } => {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(vertices.clone(), instances.clone());
            }
            VertexSource::VertexIndexBuffer {
                vertex_buffer,
                index_buffer,
                indices,
                instances,
            } => {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(indices.clone(), 0, instances.clone());
            }
        }
    }
}

// TODO: make a builder?
fn make_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    shader_module: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    vertex_buffer_layout: wgpu::VertexBufferLayout,
    blend: Option<wgpu::BlendState>,
    label: &str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[vertex_buffer_layout],
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
                format: texture_format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    })
}

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct SpriteParams {
    pub transform: Matrix4<f32>,
}

pub struct SpritePipeline(wgpu::RenderPipeline);
impl SpritePipeline {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("sprite.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layouts.texture],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..(mem::size_of::<SpriteParams>() as u32),
            }],
        });

        Self(make_pipeline(
            device,
            texture_format,
            shader_module,
            layout,
            PosColTexVertex::desc(),
            Some(wgpu::BlendState::ALPHA_BLENDING),
            "sprite_pipeline",
        ))
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, PosColTexVertex>,
        texture: &'a TextureBindGroup,
        transform: Matrix4<f32>,
    ) {
        render_pass.set_pipeline(&self.0);
        render_pass.set_bind_group(0, &texture.0, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[SpriteParams { transform }]),
        );
        source.draw(render_pass);
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct FillParams {
    pub transform: Matrix4<f32>,
    pub color: Vector4<f32>,
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

        Self(make_pipeline(
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
        transform: Matrix4<f32>,
        color: Vector4<f32>,
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

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct TextParams {
    pub transform: Matrix4<f32>,
    pub time: f32,
}

pub struct TextPipeline(wgpu::RenderPipeline);
impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layouts.texture],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..mem::size_of::<TextParams>() as u32,
            }],
        });

        let desc = TextVertex::desc();

        Self(make_pipeline(
            device,
            texture_format,
            shader_module,
            layout,
            desc,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            "text_pipeline",
        ))
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        source: VertexSource<'a, TextVertex>,
        texture: &'a TextureBindGroup,
        transform: Matrix4<f32>,
        time: f32,
    ) {
        render_pass.set_pipeline(&self.0);
        render_pass.set_bind_group(0, &texture.0, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[TextParams { transform, time }]),
        );
        source.draw(render_pass);
    }
}

pub struct Pipelines {
    pub sprite: SpritePipeline,
    pub fill: FillPipeline,
    pub text: TextPipeline,
    // those are pipelines using screen's texture format (not our preferred RGBA format)
    // they are only used for the final render pass
    pub sprite_screen: SpritePipeline,
    pub fill_screen: FillPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        surface_texture_format: wgpu::TextureFormat,
    ) -> Pipelines {
        Pipelines {
            sprite: SpritePipeline::new(device, bind_group_layouts, render::TEXTURE_FORMAT),
            fill: FillPipeline::new(device, bind_group_layouts, render::TEXTURE_FORMAT),
            text: TextPipeline::new(device, bind_group_layouts, render::TEXTURE_FORMAT),

            sprite_screen: SpritePipeline::new(device, bind_group_layouts, surface_texture_format),
            fill_screen: FillPipeline::new(device, bind_group_layouts, surface_texture_format),
        }
    }
}
