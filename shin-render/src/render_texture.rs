use std::sync::Arc;

use shin_render_shader_types::texture::{
    TextureSampler, TextureSource, TextureTarget, TextureTargetKind,
};

use crate::{
    resize::{CanvasSize, ResizeHandle},
    resizeable_texture::ResizeableTexture,
    TEXTURE_FORMAT,
};

#[derive(Debug)]
pub struct RenderTexture {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    inner_texture: ResizeableTexture<CanvasSize>,
    // an idea: maybe we should store a sampler enum instead of actual object?
    // we would almost 100% only need a linear filtering sampler, and maaaybe a nearest one
    sampler: TextureSampler,
    label: String,
}

impl RenderTexture {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        resize_handle: ResizeHandle<CanvasSize>,
        label: Option<String>,
    ) -> Self {
        let sampler = TextureSampler::Linear;

        let label = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let inner_texture = ResizeableTexture::new(
            device.clone(),
            Some(label.clone()),
            TEXTURE_FORMAT,
            resize_handle,
        );

        Self {
            device,
            queue,
            inner_texture,
            sampler,
            label,
        }
    }

    pub fn as_texture_source(&self) -> TextureSource {
        TextureSource {
            view: &self.inner_texture.get_view(),
            sampler: self.sampler,
        }
    }

    pub fn as_texture_target(&mut self) -> TextureTarget {
        TextureTarget {
            kind: TextureTargetKind::RenderTexture,
            view: &self.inner_texture.resize_and_get_view(),
        }
    }
}

impl Clone for RenderTexture {
    fn clone(&self) -> Self {
        let resize_handle = self.inner_texture.get_resize_handle();

        let size = resize_handle.get_without_update().into();

        let new_texture = ResizeableTexture::new_with_size(
            self.device.clone(),
            Some(self.label.clone()),
            TEXTURE_FORMAT,
            size,
            resize_handle,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderTexture::clone"),
            });

        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: self.inner_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &new_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            inner_texture: new_texture,
            sampler: self.sampler,
            label: self.label.clone(),
        }
    }
}
