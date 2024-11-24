use std::sync::Arc;

use dpi::PhysicalSize;

use crate::resize::{ResizeHandle, SizeAspect};

pub struct ResizeableTexture<Aspect: SizeAspect> {
    device: Arc<wgpu::Device>,
    label: Option<String>,
    texture: (wgpu::Texture, wgpu::TextureView),
    format: wgpu::TextureFormat,
    resize_handle: ResizeHandle<Aspect>,
}

impl<Aspect: SizeAspect> ResizeableTexture<Aspect> {
    fn create_texture(
        device: &wgpu::Device,
        label: Option<&str>,
        format: wgpu::TextureFormat,
        size: PhysicalSize<u32>,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            ..wgpu::TextureViewDescriptor::default()
        });

        (texture, view)
    }

    pub fn new(
        device: Arc<wgpu::Device>,
        label: Option<&str>,
        format: wgpu::TextureFormat,
        mut resize_handle: ResizeHandle<Aspect>,
    ) -> Self {
        let texture = Self::create_texture(&device, label, format, resize_handle.get().into());

        let label = label.map(|s| s.to_string());

        Self {
            device,
            label,
            texture,
            format,
            resize_handle,
        }
    }

    pub fn get_view(&mut self) -> &wgpu::TextureView {
        if let Some(new_size) = self.resize_handle.update() {
            self.texture = Self::create_texture(
                &self.device,
                self.label.as_deref(),
                self.format,
                new_size.into(),
            );
        }

        &self.texture.1
    }
}
