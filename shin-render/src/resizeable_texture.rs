use dpi::PhysicalSize;

use crate::resize::{ResizeHandle, SizeAspect};

#[derive(Debug)]
pub struct ResizeableTexture<Aspect: SizeAspect> {
    device: wgpu::Device,
    label: String,
    texture: (wgpu::Texture, wgpu::TextureView),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    resize_handle: ResizeHandle<Aspect>,
}

impl<Aspect: SizeAspect> ResizeableTexture<Aspect> {
    fn create_texture(
        device: &wgpu::Device,
        label: &str,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        size: PhysicalSize<u32>,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{}/view", label)),
            ..wgpu::TextureViewDescriptor::default()
        });

        (texture, view)
    }

    pub fn new_with_size(
        device: wgpu::Device,
        label: Option<String>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        size: PhysicalSize<u32>,
        resize_handle: ResizeHandle<Aspect>,
    ) -> Self {
        let label = label.unwrap_or_else(|| "unnamed".to_string());

        let texture = Self::create_texture(&device, &label, format, usage, size);

        Self {
            device,
            label,
            texture,
            format,
            usage,
            resize_handle,
        }
    }

    pub fn new(
        device: wgpu::Device,
        label: Option<String>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mut resize_handle: ResizeHandle<Aspect>,
    ) -> Self {
        let size = resize_handle.get().into();

        Self::new_with_size(device, label, format, usage, size, resize_handle)
    }

    pub fn get_texture(&self) -> &wgpu::Texture {
        &self.texture.0
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }

    pub fn resize_and_get_view(&mut self) -> &wgpu::TextureView {
        if let Some(new_size) = self.resize_handle.update() {
            self.texture = Self::create_texture(
                &self.device,
                &self.label,
                self.format,
                self.usage,
                new_size.into(),
            );
        }

        &self.texture.1
    }

    pub fn get_resize_handle(&self) -> ResizeHandle<Aspect> {
        self.resize_handle.clone()
    }
}
