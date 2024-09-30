use std::sync::Arc;

use crate::resize::{SurfaceResizeHandle, SurfaceSize};

pub struct ResizeableTexture {
    device: Arc<wgpu::Device>,
    texture: (wgpu::Texture, wgpu::TextureView),
    format: wgpu::TextureFormat,
    resize_handle: SurfaceResizeHandle,
}

impl ResizeableTexture {
    fn create_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: SurfaceSize,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size.into(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        mut resize_handle: SurfaceResizeHandle,
    ) -> Self {
        let texture = Self::create_texture(&device, format, resize_handle.get());

        Self {
            device,
            texture,
            format,
            resize_handle,
        }
    }

    pub fn get_view(&mut self) -> &wgpu::TextureView {
        if let Some(new_size) = self.resize_handle.update() {
            self.texture = Self::create_texture(&self.device, self.format, new_size);
        }

        &self.texture.1
    }
}
