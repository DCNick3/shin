use std::{borrow::Cow, marker::PhantomData};

pub use shin_render_shader_types as types;
use shin_render_shader_types::{
    buffer::{DynamicBuffer, VertexSource, VertexSourceInfo},
    texture::TextureBindGroup,
    vertices::VertexType,
};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderBindingGroupDescriptor {
    Texture,
    Uniform { size: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderDescriptor {
    pub name: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    pub spirv: &'static [u32],
    #[cfg(target_arch = "wasm32")]
    pub wgsl: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
    pub bind_groups: &'static [ShaderBindingGroupDescriptor],
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
}

pub enum ShaderBindGroupLayout {
    Texture,
    Uniform(wgpu::BindGroupLayout),
}

/// Stores the shader-independent parts of a render pipeline: a shader module and pipeline layout.
pub struct ShaderContext {
    pub shader_descriptor: ShaderDescriptor,
    pub bind_group_layouts: Vec<ShaderBindGroupLayout>,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
}

impl ShaderDescriptor {
    pub fn create_shader_context(
        &self,
        device: &wgpu::Device,
        texture_bind_group_layout: &TextureBindGroupLayout,
    ) -> ShaderContext {
        let mut bind_group_layouts_owned = Vec::new();

        for (bind_group, index) in self.bind_groups.iter().zip(0..) {
            match *bind_group {
                ShaderBindingGroupDescriptor::Texture => {
                    bind_group_layouts_owned.push(ShaderBindGroupLayout::Texture);
                }
                ShaderBindingGroupDescriptor::Uniform { size } => {
                    bind_group_layouts_owned.push(ShaderBindGroupLayout::Uniform(
                        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some(&format!(
                                "{} uniform bind group {} layout",
                                self.name, index
                            )),
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: Some(
                                        wgpu::BufferSize::new(size as u64).unwrap(),
                                    ),
                                },
                                count: None,
                            }],
                        }),
                    ));
                }
            }
        }

        let bind_group_layouts = bind_group_layouts_owned
            .iter()
            .map(|bind_group_layout| match bind_group_layout {
                ShaderBindGroupLayout::Texture => &texture_bind_group_layout.0,
                ShaderBindGroupLayout::Uniform(bind_group_layout) => bind_group_layout,
            })
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} pipeline layout", self.name)),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        #[cfg(not(target_arch = "wasm32"))]
        // SAFETY: well, naga spit it out, so it should be good, right?
        let shader_module = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: Some(&format!("{} shader module", self.name)),
                source: self.spirv.into(),
            })
        };
        #[cfg(target_arch = "wasm32")]
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} shader module", self.name)),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(self.wgsl)),
        });

        ShaderContext {
            shader_descriptor: self.clone(),
            bind_group_layouts: bind_group_layouts_owned,
            pipeline_layout,
            shader_module,
        }
    }
}

pub trait Shader {
    const NAME: ShaderName;
    const DESCRIPTOR: ShaderDescriptor;

    type Bindings;
    type Vertex: VertexType;

    fn set_bindings(
        device: &wgpu::Device,
        dynamic_buffer: &mut DynamicBuffer,
        bind_group_layouts: &[ShaderBindGroupLayout],
        render_pass: &mut wgpu::RenderPass,
        bindings: &Self::Bindings,
    );
}

pub struct TypedRenderPipeline<'a, S: Shader> {
    context: &'a ShaderContext,
    pipeline: &'a wgpu::RenderPipeline,
    phantom: PhantomData<S>,
}

impl<'a, S: Shader> TypedRenderPipeline<'a, S> {
    pub fn new(context: &'a ShaderContext, pipeline: &'a wgpu::RenderPipeline) -> Self {
        Self {
            context,
            pipeline,
            phantom: PhantomData,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        dynamic_buffer: &mut DynamicBuffer,
        render_pass: &mut wgpu::RenderPass,
        bindings: &S::Bindings,
        vertices: VertexSource<S::Vertex>,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        S::set_bindings(
            device,
            dynamic_buffer,
            &self.context.bind_group_layouts,
            render_pass,
            bindings,
        );
        vertices.bind(render_pass);
        match vertices.info() {
            VertexSourceInfo::VertexBuffer { vertex_count } => {
                render_pass.draw(0..vertex_count, 0..1);
            }
            VertexSourceInfo::VertexAndIndexBuffer { index_count } => {
                render_pass.draw_indexed(0..index_count, 0, 0..1);
            }
        }
    }
}

mod shaders {
    include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
}

pub use shaders::*;
use shin_render_shader_types::texture::TextureBindGroupLayout;
