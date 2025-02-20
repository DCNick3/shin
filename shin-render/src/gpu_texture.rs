use dpi::PhysicalSize;
use glam::{vec2, Vec2};
use shin_render_shader_types::texture::{TextureSampler, TextureSource};
use wgpu::TextureDimension;

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
        mip_levels: u32,
    ) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_levels,
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
        data: &[u8],
    ) -> Self {
        Self::new_static_with_mip_data(device, queue, label, size, format, &[data])
    }

    pub fn new_static_with_mip_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        data: &[&[u8]],
    ) -> Self {
        let mip_levels = data.len() as u32;
        let mut desc = Self::make_descriptor(label, size, format, TextureKind::Static, mip_levels);
        // Implicitly add the COPY_DST usage
        desc.usage |= wgpu::TextureUsages::COPY_DST;
        let texture = device.create_texture(&desc);

        assert_eq!(desc.format.block_dimensions(), (1, 1));
        assert_eq!(desc.array_layer_count(), 1);
        let bpp = desc.format.block_copy_size(None).unwrap();

        for (mip_level, &mip_data) in (0..).zip(data) {
            let mip_size = desc.mip_level_size(mip_level).unwrap();
            assert_eq!(mip_size.depth_or_array_layers, 1);

            let bytes_per_row = mip_size.width * bpp;

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                mip_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(mip_size.height),
                },
                mip_size,
            );
        }

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

    pub fn new_static_from_rgba_image(
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
            image.as_ref(),
        )
    }

    pub fn new_static_from_gray_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        image: &image::GrayImage,
    ) -> Self {
        // NB: no sRGB, because that's how the original code did it
        let format = wgpu::TextureFormat::R8Unorm;

        Self::new_static_with_data(
            device,
            queue,
            label,
            image.dimensions().into(),
            format,
            image.as_ref(),
        )
    }

    pub fn new_static_from_gray_mipped_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        mip_levels: &[&image::GrayImage],
    ) -> Self {
        // NB: no sRGB, because that's how the original code did it
        let format = wgpu::TextureFormat::R8Unorm;

        let size = mip_levels[0].dimensions().into();

        let data = mip_levels
            .iter()
            .map(|&image| image.as_ref())
            .collect::<Vec<_>>();

        Self::new_static_with_mip_data(device, queue, label, size, format, data.as_slice())
    }

    pub fn new_empty(
        device: &wgpu::Device,
        label: Option<&str>,
        size: PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        kind: TextureKind,
    ) -> Self {
        let texture = device.create_texture(&Self::make_descriptor(label, size, format, kind, 1));
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

    pub fn size_vec(&self) -> Vec2 {
        vec2(self.texture.width() as f32, self.texture.height() as f32)
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
