use crate::render::bind_group_layouts::BindGroupLayouts;
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
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("picture_texture_{:08x}", picture.picture_id)),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });
    texture
}

pub struct GpuPicture {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub picture_id: u32,
}

impl GpuPicture {
    pub fn load(
        device: &wgpu::Device,
        bind_group_layouts: &BindGroupLayouts,
        queue: &mut wgpu::Queue,
        picture: Picture,
    ) -> GpuPicture {
        let texture = make_texture(device, &picture);
        queue.write_texture(
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("picture_sampler_{:08x}", picture.picture_id)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("picture_bind_group_{:08x}", picture.picture_id)),
            layout: &bind_group_layouts.picture,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

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

pub fn make_picture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("picture_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}
