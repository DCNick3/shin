use std::marker::PhantomData;

pub use shin_render_shader_types as types;
use shin_render_shader_types::{
    buffer::{DynamicBufferBackend, VertexSource, VertexSourceInfo},
    vertices::VertexType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderBindingGroupDescriptor {
    Texture {
        texture_binding: u32,
        sampler_binding: u32,
    },
    Uniform {
        binding: u32,
        size: u32,
    },
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

// pub enum ShaderBindGroupLayout {
//     Texture,
//     Uniform(wgpu::BindGroupLayout),
// }

/// Stores the shader-independent parts of a render pipeline: a shader module and pipeline layout.
pub struct ShaderContext {
    pub shader_descriptor: ShaderDescriptor,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
}

impl ShaderDescriptor {
    pub fn create_shader_context(&self, device: &wgpu::Device) -> ShaderContext {
        let mut entries = Vec::new();
        for bind_group in self.bind_groups.iter() {
            match bind_group {
                &ShaderBindingGroupDescriptor::Texture {
                    texture_binding,
                    sampler_binding,
                } => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: texture_binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    });
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: sampler_binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        // kinda unfortunate, we may want to change this dynamically?
                        // probably won't matter for most cases though
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    });
                }
                &ShaderBindingGroupDescriptor::Uniform { binding, size } => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(wgpu::BufferSize::new(size as u64).unwrap()),
                        },
                        count: None,
                    });
                }
            }
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} bind group layout", self.name)),
            entries: &entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} pipeline layout", self.name)),
            bind_group_layouts: &[&bind_group_layout],
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
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(self.wgsl)),
        });

        ShaderContext {
            shader_descriptor: self.clone(),
            bind_group_layout,
            pipeline_layout,
            shader_module,
        }
    }
}

pub trait Shader {
    const NAME: ShaderName;
    const DESCRIPTOR: ShaderDescriptor;

    type Bindings<'a>: 'a;
    type Vertex: VertexType;

    fn set_bindings(
        device: &wgpu::Device,
        dynamic_buffer: &mut impl DynamicBufferBackend,
        bind_group_layout: &wgpu::BindGroupLayout,
        render_pass: &mut wgpu::RenderPass,
        bindings: Self::Bindings<'_>,
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
        dynamic_buffer: &mut impl DynamicBufferBackend,
        render_pass: &mut wgpu::RenderPass,
        bindings: S::Bindings<'_>,
        vertices: VertexSource<S::Vertex>,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        S::set_bindings(
            device,
            dynamic_buffer,
            &self.context.bind_group_layout,
            render_pass,
            bindings,
        );
        vertices.bind(dynamic_buffer, render_pass);
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
