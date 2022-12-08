use crate::render::{self, GpuCommonResources, TextureBindGroup};
use anyhow::Result;
use shin_core::format::picture::SimpleMergedPicture;
use std::num::NonZeroU32;

pub type Picture = SimpleMergedPicture;

pub fn load_picture(bytes: &[u8]) -> Result<Picture> {
    shin_core::format::picture::read_picture::<Picture>(bytes, ())
}

fn make_texture(device: &wgpu::Device, picture: &Picture) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: picture.image.width(),
        height: picture.image.height(),
        depth_or_array_layers: 1,
    };

    assert_eq!(
        render::TEXTURE_FORMAT,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        "Only Rgba8UnormSrgb is supported for now"
    );

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("picture_texture_{:08x}", picture.picture_id)),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: render::TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });
    texture
}

pub struct GpuPicture {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub bind_group: TextureBindGroup,
    pub width: u32,
    pub height: u32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub picture_id: u32,
}

impl GpuPicture {
    pub fn load(resources: &GpuCommonResources, picture: Picture) -> GpuPicture {
        let texture = make_texture(&resources.device, &picture);
        resources.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &picture.image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * picture.image.width()),
                rows_per_image: NonZeroU32::new(picture.image.height()),
            },
            wgpu::Extent3d {
                width: picture.image.width(),
                height: picture.image.height(),
                depth_or_array_layers: 1,
            },
        );

        let sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("picture_sampler_{:08x}", picture.picture_id)),
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
            &resources,
            &texture_view,
            &sampler,
            Some(&format!("picture_bind_group_{:08x}", picture.picture_id)),
        );

        GpuPicture {
            texture,
            sampler,
            bind_group,
            width: picture.image.width(),
            height: picture.image.height(),
            origin_x: picture.origin_x as f32,
            origin_y: picture.origin_y as f32,
            picture_id: picture.picture_id,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
