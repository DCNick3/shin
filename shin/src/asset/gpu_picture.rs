use crate::render;
use crate::render::{GpuCommonResources, SpriteVertexBuffer, TextureBindGroup};
use cgmath::{Vector2, Vector4};
use image::RgbaImage;
use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::num::NonZeroU32;

pub struct LazyGpuImage {
    image: RgbaImage,
    origin: Vector2<f32>,
    label: Option<String>,
    gpu_image: OnceCell<GpuImage>,
}

impl LazyGpuImage {
    pub fn new(image: RgbaImage, origin: Vector2<f32>, label: Option<&str>) -> Self {
        Self {
            image,
            origin,
            label: label.map(|s| s.to_owned()),
            gpu_image: OnceCell::new(),
        }
    }

    pub fn gpu_image(&self, resources: &GpuCommonResources) -> &GpuImage {
        self.gpu_image.get_or_init(|| {
            GpuImage::load(resources, &self.image, self.origin, self.label.as_deref())
        })
    }
}

/// Gpu picture, ready to be drawn
/// Includes a texture, a sampler, a bind group, and a vertex buffer
pub struct GpuImage {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub bind_group: TextureBindGroup,
    pub vertex_buffer: SpriteVertexBuffer,
    pub width: u32,
    pub height: u32,
}

impl GpuImage {
    pub fn load(
        resources: &GpuCommonResources,
        image: &RgbaImage,
        origin: Vector2<f32>,
        label: Option<&str>,
    ) -> GpuImage {
        let label = label
            .map(|s| Cow::from(s.to_owned()))
            .unwrap_or_else(|| Cow::from("Unnamed GpuPicture"));

        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        assert_eq!(
            render::TEXTURE_FORMAT,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "Only Rgba8UnormSrgb is supported for now"
        );

        let texture = resources.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} Texture", label)),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: render::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        resources.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * image.width()),
                rows_per_image: NonZeroU32::new(image.height()),
            },
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
        );

        let sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = TextureBindGroup::new(
            resources,
            &texture_view,
            &sampler,
            Some(&format!("{} BindGroup", label)),
        );

        let origin_translate = -origin.extend(0.0);

        let vertex_buffer = SpriteVertexBuffer::new(
            resources,
            (
                origin_translate.x,
                origin_translate.y,
                origin_translate.x + image.width() as f32,
                origin_translate.y + image.height() as f32,
            ),
            // TODO: do we even want colored vertices?..
            Vector4::new(1.0, 1.0, 1.0, 1.0),
        );

        GpuImage {
            texture,
            sampler,
            bind_group,
            vertex_buffer,
            width: image.width(),
            height: image.height(),
        }
    }
}
