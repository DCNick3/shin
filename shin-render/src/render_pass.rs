use glam::{vec3, vec4};
use shin_core::primitives::color::{FloatColor4, UnormColor};
use shin_render_shader_types::{
    buffer::VertexSource,
    texture::{DepthStencilTarget, TextureSamplerStore, TextureTarget, TextureTargetKind},
    uniforms::{
        ClearUniformParams, FillUniformParams, FontBorderUniformParams, FontUniformParams,
        LayerUniformParams, MaskUniformParams, MovieUniformParams, SpriteUniformParams,
        WiperDefaultUniformParams, WiperMaskUniformParams,
    },
    vertices::PosVertex,
};
use shin_render_shaders::{
    Clear, ClearBindings, Fill, FillBindings, Font, FontBindings, FontBorder, FontBorderBindings,
    Layer, LayerBindings, Mask, MaskBindings, Movie, MovieBindings, Shader, Sprite, SpriteBindings,
    WiperDefault, WiperDefaultBindings, WiperMask, WiperMaskBindings,
};

use crate::{
    ColorBlendType, CullFace, DepthFunction, DepthState, DepthStencilState, DrawPrimitive,
    RenderProgramWithArguments, RenderRequest, RenderRequestBuilder, StencilFunction,
    StencilOperation, StencilPipelineState, StencilState,
    dynamic_buffer::DynamicBuffer,
    pipelines::{PipelineStorage, PipelineStorageKey},
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

    fn run_impl<S: Shader>(
        &mut self,
        key: PipelineStorageKey,
        bindings: S::Bindings<'_>,
        vertices: VertexSource<S::Vertex>,
    ) {
        self.pipeline_storage.get::<S>(key).render(
            self.device,
            self.dynamic_buffer,
            self.sampler_store,
            &mut self.pass,
            bindings,
            vertices,
        )
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
            RenderProgramWithArguments::Clear { vertices, color } => self.run_impl::<Clear>(
                key,
                ClearBindings {
                    params: &ClearUniformParams { color },
                },
                vertices,
            ),
            RenderProgramWithArguments::Fill {
                vertices,
                transform,
            } => self.run_impl::<Fill>(
                key,
                FillBindings {
                    params: &FillUniformParams { transform },
                },
                vertices,
            ),
            RenderProgramWithArguments::Sprite {
                vertices,
                sprite,
                transform,
            } => self.run_impl::<Sprite>(
                key,
                SpriteBindings {
                    params: &SpriteUniformParams { transform },
                    sprite,
                },
                vertices,
            ),
            RenderProgramWithArguments::Font {
                vertices,
                glyph,
                transform,
                color1,
                color2,
            } => self.run_impl::<Font>(
                key,
                FontBindings {
                    params: &FontUniformParams {
                        transform,
                        color1,
                        color2,
                    },
                    glyph,
                },
                vertices,
            ),
            RenderProgramWithArguments::FontBorder {
                vertices,
                glyph,
                transform,
                distances,
                color,
            } => self.run_impl::<FontBorder>(
                key,
                FontBorderBindings {
                    params: &FontBorderUniformParams {
                        transform,
                        // [Vec2; 8] -> [Vec4; 4]
                        dist: bytemuck::cast(distances),
                        color,
                    },
                    glyph,
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
            } => self.run_impl::<Layer>(
                key,
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
            RenderProgramWithArguments::Mask {
                fragment_shader,
                vertices,
                texture,
                mask,
                transform,
                color_multiplier,
                fragment_shader_param,
                minmax,
            } => self.run_impl::<Mask>(
                key,
                MaskBindings {
                    params: &MaskUniformParams {
                        transform,
                        color: color_multiplier,
                        fragment_param: fragment_shader_param,
                        minmax: minmax.extend(0.0).extend(0.0),
                        fragment_operation: fragment_shader as u32,
                    },
                    texture,
                    mask,
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
            } => self.run_impl::<Movie>(
                key,
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

            RenderProgramWithArguments::WiperDefault {
                vertices,
                texture_source,
                texture_target,
                transform,
                alpha,
            } => self.run_impl::<WiperDefault>(
                key,
                WiperDefaultBindings {
                    params: &WiperDefaultUniformParams {
                        transform,
                        alpha: vec4(alpha, 0.0, 0.0, 0.0),
                    },
                    source: texture_source,
                    target: texture_target,
                },
                vertices,
            ),

            RenderProgramWithArguments::WiperMask {
                vertices,
                texture_source,
                texture_target,
                texture_mask,
                transform,
                minmax,
            } => self.run_impl::<WiperMask>(
                key,
                WiperMaskBindings {
                    params: &WiperMaskUniformParams {
                        transform,
                        minmax: minmax.extend(0.0).extend(0.0),
                    },
                    source: texture_source,
                    target: texture_target,
                    mask: texture_mask,
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
        // draw a single full-screen triangle
        // https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2
        let vertices = &[
            PosVertex {
                position: vec3(-1.0, -1.0, z),
            },
            PosVertex {
                position: vec3(3.0, -1.0, z),
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

        self.push_debug(&format!("clear[{:?}, {:?}, {:?}]", color, stencil, depth));
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
        self.pop_debug();
    }
}
