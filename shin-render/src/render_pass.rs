use shin_render_shader_types::{
    buffer::DynamicBuffer,
    uniforms::{ClearUniformParams, FillUniformParams, SpriteUniformParams},
};
use shin_render_shaders::{Clear, ClearBindings, Fill, FillBindings, Sprite, SpriteBindings};

use crate::{
    pipelines::{PipelineStorage, PipelineStorageKey},
    RenderProgramWithArguments, RenderRequest,
};

pub struct RenderPass<'pipelines, 'dynbuffer, 'device, 'encoder> {
    pipeline_storage: &'pipelines mut PipelineStorage,
    dynamic_buffer: &'dynbuffer mut DynamicBuffer,
    device: &'device wgpu::Device,
    pass: wgpu::RenderPass<'encoder>,
}

impl<'pipelines, 'dynbuffer, 'device, 'encoder>
    RenderPass<'pipelines, 'dynbuffer, 'device, 'encoder>
{
    pub fn new(
        pipeline_storage: &'pipelines mut PipelineStorage,
        dynamic_buffer: &'dynbuffer mut DynamicBuffer,
        device: &'device wgpu::Device,
        encoder: &'encoder mut wgpu::CommandEncoder,
        target_color: &wgpu::TextureView,
        target_depth_stencil: &wgpu::TextureView,
    ) -> Self {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_color,
                resolve_target: None,
                ops: wgpu::Operations {
                    // NOTE: potential incompatibility, shin _might_ not perform a clear when changing a render target
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target_depth_stencil,
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

        Self {
            pipeline_storage,
            dynamic_buffer,
            device,
            pass,
        }
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
                    pass,
                    &ClearBindings {
                        params: ClearUniformParams { color },
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
                    pass,
                    &FillBindings {
                        params: FillUniformParams { transform },
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
                pass,
                &SpriteBindings {
                    params: SpriteUniformParams { transform },
                    sprite,
                },
                vertices,
            ),

            _ => todo!(),
        }
    }
}
