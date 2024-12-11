use glam::vec3;
use shin_render_shader_types::{
    buffer::VertexSource,
    texture::{DepthStencilTarget, TextureSamplerStore, TextureTarget, TextureTargetKind},
    uniforms::{
        ClearUniformParams, FillUniformParams, LayerUniformParams, MovieUniformParams,
        SpriteUniformParams,
    },
    vertices::{FloatColor4, PosVertex, UnormColor},
};
use shin_render_shaders::{
    Clear, ClearBindings, Fill, FillBindings, Layer, LayerBindings, Movie, MovieBindings, Sprite,
    SpriteBindings,
};

use crate::{
    dynamic_buffer::DynamicBuffer,
    pipelines::{PipelineStorage, PipelineStorageKey},
    ColorBlendType, CullFace, DepthFunction, DepthState, DepthStencilState, DrawPrimitive,
    RenderProgramWithArguments, RenderRequest, RenderRequestBuilder, StencilFunction,
    StencilOperation, StencilPipelineState, StencilState,
};

pub struct RenderPass<'pipelines, 'dynbuffer, 'sampler, 'device, 'encoder> {
    pipeline_storage: &'pipelines mut PipelineStorage,
    dynamic_buffer: &'dynbuffer mut DynamicBuffer,
    sampler_store: &'sampler TextureSamplerStore,
    target_kind: TextureTargetKind,
    device: &'device wgpu::Device,
    pass: wgpu::RenderPass<'encoder>,
}

impl<'pipelines, 'dynbuffer, 'sampler, 'device, 'encoder>
    RenderPass<'pipelines, 'dynbuffer, 'sampler, 'device, 'encoder>
{
    pub fn new(
        pipeline_storage: &'pipelines mut PipelineStorage,
        dynamic_buffer: &'dynbuffer mut DynamicBuffer,
        sampler_store: &'sampler TextureSamplerStore,
        device: &'device wgpu::Device,
        encoder: &'encoder mut wgpu::CommandEncoder,
        target_color: TextureTarget,
        target_depth_stencil: DepthStencilTarget,
        viewport: Option<(f32, f32, f32, f32)>,
    ) -> Self {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_color.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // NOTE: potential incompatibility, shin _might_ not perform a clear when changing a render target
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target_depth_stencil.view,
                depth_ops: Some(wgpu::Operations {
                    // NOTE: potential incompatibility, shin _might_ not perform a clear when changing a render target
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    // NOTE: potential incompatibility, shin _might_ not perform a clear when changing a render target
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some((x, y, width, height)) = viewport {
            pass.set_viewport(x, y, width, height, 0.0, 1.0);
        }

        Self {
            pipeline_storage,
            dynamic_buffer,
            sampler_store,
            target_kind: target_color.kind,
            device,
            pass,
        }
    }

    pub fn push_debug(&mut self, label: &str) {
        self.pass.push_debug_group(label)
    }

    pub fn pop_debug(&mut self) {
        self.pass.pop_debug_group()
    }

    pub fn run(&mut self, request: RenderRequest) {
        let pass = &mut self.pass;

        let RenderRequest {
            depth_stencil,
            color_blend_type,
            cull_faces,
            primitive,
            program,
        } = request;

        let (depth_stencil, stencil_reference) = depth_stencil.into_pipeline_parts();

        let key = PipelineStorageKey {
            target_kind: self.target_kind,
            draw_primitive: primitive,
            cull_face: cull_faces,
            blend_type: color_blend_type,
            depth_stencil,
        };
        pass.set_stencil_reference(stencil_reference as u32);

        match program {
            RenderProgramWithArguments::Clear { vertices, color } => {
                self.pipeline_storage.get::<Clear>(key).render(
                    self.device,
                    self.dynamic_buffer,
                    self.sampler_store,
                    pass,
                    ClearBindings {
                        params: &ClearUniformParams { color },
                    },
                    vertices,
                );
            }
            RenderProgramWithArguments::Fill {
                vertices,
                transform,
            } => {
                self.pipeline_storage.get::<Fill>(key).render(
                    self.device,
                    self.dynamic_buffer,
                    self.sampler_store,
                    pass,
                    FillBindings {
                        params: &FillUniformParams { transform },
                    },
                    vertices,
                );
            }
            RenderProgramWithArguments::Sprite {
                vertices,
                sprite,
                transform,
            } => self.pipeline_storage.get::<Sprite>(key).render(
                self.device,
                self.dynamic_buffer,
                self.sampler_store,
                pass,
                SpriteBindings {
                    params: &SpriteUniformParams { transform },
                    sprite,
                },
                vertices,
            ),

            RenderProgramWithArguments::Layer {
                output_kind,
                fragment_shader,
                vertices,
                texture,
                transform,
                color_multiplier,
                fragment_shader_param,
            } => self.pipeline_storage.get::<Layer>(key).render(
                self.device,
                self.dynamic_buffer,
                self.sampler_store,
                pass,
                LayerBindings {
                    params: &LayerUniformParams {
                        transform,
                        color: color_multiplier,
                        fragment_param: fragment_shader_param,
                        output_type: output_kind as u32,
                        fragment_operation: fragment_shader as u32,
                    },
                    texture,
                },
                vertices,
            ),

            RenderProgramWithArguments::Movie {
                vertices,
                texture_luma,
                texture_chroma,
                transform,
                color_bias,
                color_transform,
            } => self.pipeline_storage.get::<Movie>(key).render(
                self.device,
                self.dynamic_buffer,
                self.sampler_store,
                pass,
                MovieBindings {
                    params: &MovieUniformParams {
                        transform,
                        color_bias,
                        color_transform,
                    },
                    luma: texture_luma,
                    chroma: texture_chroma,
                },
                vertices,
            ),

            _ => todo!(),
        }
    }

    pub fn clear(&mut self, color: Option<UnormColor>, stencil: Option<u8>, depth: Option<f32>) {
        let z = match depth {
            Some(z) => z + z - 1.0,
            None => 1.0,
        };
        let vertices = &[
            PosVertex {
                position: vec3(-1.0, 1.0, z),
            },
            PosVertex {
                position: vec3(3.0, 1.0, z),
            },
            PosVertex {
                position: vec3(-1.0, 3.0, z),
            },
        ];
        let color_param = match color {
            Some(color) => FloatColor4::from_unorm(color),
            None => FloatColor4::BLACK,
        };
        let stencil_param = stencil.unwrap_or(1);
        let stencil_operation = if stencil.is_some() {
            StencilOperation::Replace
        } else {
            StencilOperation::Keep
        };

        self.run(
            RenderRequestBuilder::new()
                .color_blend_type(if color.is_some() {
                    ColorBlendType::Opaque
                } else {
                    ColorBlendType::NoColor
                })
                .depth_stencil(DepthStencilState {
                    depth: DepthState {
                        function: DepthFunction::Always,
                        write_enable: depth.is_some(),
                    },
                    stencil: StencilState {
                        pipeline: StencilPipelineState {
                            function: StencilFunction::Always,
                            stencil_fail_operation: stencil_operation,
                            depth_fail_operation: stencil_operation,
                            pass_operation: stencil_operation,
                            ..Default::default()
                        },
                        stencil_reference: stencil_param,
                    },
                })
                .cull_faces(CullFace::None)
                .build(
                    RenderProgramWithArguments::Clear {
                        vertices: VertexSource::VertexData { vertices },
                        color: color_param,
                    },
                    DrawPrimitive::Triangles,
                ),
        );
    }
}
