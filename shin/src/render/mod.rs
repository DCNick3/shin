pub mod bind_group_layouts;
mod camera;
mod picture_layer;
mod pillarbox;
mod pipelines;
mod window;

use crate::render::bind_group_layouts::BindGroupLayouts;
use crate::render::pipelines::{CommonBinds, Pipelines};
pub use window::run;

pub struct RenderContext<'cmd, 'pass> {
    pub device: &'cmd wgpu::Device,
    pub queue: &'cmd wgpu::Queue,
    pub render_pass: &'pass mut wgpu::RenderPass<'cmd>,
    pub pipelines: &'cmd Pipelines,
    pub common_binds: &'cmd CommonBinds<'cmd>,
    pub bind_group_layouts: &'cmd BindGroupLayouts,
}

impl<'cmd, 'pass> RenderContext<'cmd, 'pass> {
    pub fn render_to(&mut self, target: &RenderTarget, render_fn: impl FnOnce(&mut RenderContext)) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderTarget encoder"),
            });

        let mut render_pass: wgpu::RenderPass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RenderTarget render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        let mut new_context: RenderContext = RenderContext {
            device: self.device,
            queue: self.queue,
            render_pass: &mut render_pass,
            pipelines: self.pipelines,
            common_binds: self.common_binds,
            bind_group_layouts: self.bind_group_layouts,
        };

        render_fn(&mut new_context);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render(&mut self, renderable: &impl Renderable) {
        renderable.render(self);
    }
}

pub struct RenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl RenderTarget {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(device: &wgpu::Device, size: (u32, u32), label: Option<&str>) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RenderTarget texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn create_render_pass<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        label: Option<&str>,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }
}

pub trait Renderable {
    fn render(&self, context: &mut RenderContext);
    fn resize(&mut self, size: (u32, u32));
}
