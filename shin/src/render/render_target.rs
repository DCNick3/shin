use super::TextureBindGroup;
use crate::render::common_resources::GpuCommonResources;
use std::borrow::Cow;

pub struct RenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: TextureBindGroup,
    label: Cow<'static, str>,
}

impl RenderTarget {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(resources: &GpuCommonResources, size: (u32, u32), label: Option<&str>) -> Self {
        let label = label
            .map(|s| Cow::from(s.to_owned()))
            .unwrap_or_else(|| Cow::from("Unnamed RenderTarget"));

        let texture = resources.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} Texture", label)),
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} TextureView", label)),
            ..Default::default()
        });
        let sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = TextureBindGroup::new(
            resources,
            &view,
            &sampler,
            Some(&format!("{} TextureBindGroup", label)),
        );
        Self {
            texture,
            view,
            sampler,
            bind_group,
            label,
        }
    }

    pub fn resize(&mut self, resources: &GpuCommonResources, size: (u32, u32)) {
        self.texture = resources.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} Texture", self.label)),
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
        self.view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} TextureView", self.label)),
            ..Default::default()
        });
        self.bind_group = TextureBindGroup::new(
            resources,
            &self.view,
            &self.sampler,
            Some(&format!("{} TextureBindGroup", self.label)),
        );
    }

    pub fn begin_render_pass<'a>(
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
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn bind_group(&self) -> &TextureBindGroup {
        &self.bind_group
    }
}
