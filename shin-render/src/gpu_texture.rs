use dpi::PhysicalSize;
use shin_render_shader_types::texture::{TextureSampler, TextureSource};
use wgpu::{util::DeviceExt as _, TextureDimension};

#[derive(Debug, Copy, Clone)]
pub enum TextureKind {
    // TODO: do we want to allow `COPY_SRC` usage? This is the only other way to use a texture without changing it, but I don't think we'll need it?
    Static,
    Updatable,
}

impl TextureKind {
    fn into_usage(self) -> wgpu::TextureUsages {
        match self {
            TextureKind::Static => wgpu::TextureUsages::TEXTURE_BINDING,
            TextureKind::Updatable => {
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
            }
        }
    }
}

/// Texture that does not get used as a render target.
///
/// If you need a texture that can be used as a render target, use [`crate::resizeable_texture::ResizeableTexture`] instead.
// NB: we do not provide type-safety against wrong formats of textures
// this is kind of unfortunate, but, hopefully, not too footgunny?
#[derive(Debug)]
pub struct GpuTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: TextureSampler,
}

impl GpuTexture {
    fn make_descriptor(
        label: Option<&str>,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        kind: TextureKind,
    ) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: kind.into_usage(),
            view_formats: &[],
        }
    }

    pub fn new_static_with_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        order: wgpu::util::TextureDataOrder,
        data: &[u8],
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &Self::make_descriptor(label, size, format, TextureKind::Static),
            order,
            data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: label.map(|s| format!("{} view", s)).as_deref(),
            ..wgpu::TextureViewDescriptor::default()
        });

        let sampler = TextureSampler::Linear;

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_static_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        image: &image::RgbaImage,
    ) -> Self {
        // NB: no sRGB, because that's how the original code did it
        let format = wgpu::TextureFormat::Rgba8Unorm;

        Self::new_static_with_data(
            device,
            queue,
            label,
            image.dimensions().into(),
            format,
            wgpu::util::TextureDataOrder::LayerMajor,
            image.as_ref(),
        )
    }

    pub fn new_empty(
        device: &wgpu::Device,
        label: Option<&str>,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        kind: TextureKind,
    ) -> Self {
        let texture = device.create_texture(&Self::make_descriptor(label, size, format, kind));
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = TextureSampler::Linear;

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.texture.width(), self.texture.height())
    }

    // TODO: we should provide APIs instead of just exposing the raw wgpu texture
    // right now this is mostly used for updatable textures
    pub fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn as_source(&self) -> TextureSource {
        TextureSource {
            view: &self.view,
            sampler: self.sampler,
        }
    }
}
