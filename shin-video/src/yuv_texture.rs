use crate::h264_decoder::FrameInfo;
use shin_render::{GpuCommonResources, YuvTextureBindGroup};
use std::num::NonZeroU32;

pub struct YuvTexture {
    tex_y: wgpu::Texture,
    tex_u: wgpu::Texture,
    tex_v: wgpu::Texture,
    bind_group: YuvTextureBindGroup,
    frame_info: FrameInfo,
}

impl YuvTexture {
    pub fn new(resources: &GpuCommonResources, info: FrameInfo) -> Self {
        // note that this assumes 4:2:0 chroma subsampling is used
        // as of now, this is the only subsampling supported by openh264 crate

        assert!(matches!(info.colorspace, y4m::Colorspace::C420mpeg2));

        let device = &resources.device;

        let tex_y = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer Y Texture"),
            size: wgpu::Extent3d {
                width: info.width as u32,
                height: info.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let tex_u = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer U Texture"),
            size: wgpu::Extent3d {
                width: info.width as u32 / 2,
                height: info.height as u32 / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let tex_v = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VideoRenderer V Texture"),
            size: wgpu::Extent3d {
                width: info.width as u32 / 2,
                height: info.height as u32 / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("VideoRenderer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        let bind_group = YuvTextureBindGroup::new(
            resources,
            &tex_y.create_view(&Default::default()),
            &tex_u.create_view(&Default::default()),
            &tex_v.create_view(&Default::default()),
            &sampler,
            Some("VideoRenderer Bind Group"),
        );

        let result = Self {
            tex_y,
            tex_u,
            tex_v,
            bind_group,
            frame_info: info,
        };

        result
    }

    pub fn write_data(&self, yuv: &y4m::Frame, queue: &wgpu::Queue) {
        // note that this assumes 4:2:0 chroma subsampling is used
        // as of now, this is the only subsampling supported by openh264 crate

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_y,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.get_y_plane(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(self.frame_info.width as u32).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // Y is not subsampled
                width: self.frame_info.width as u32,
                height: self.frame_info.height as u32,
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_u,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.get_u_plane(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(self.frame_info.width as u32 / 2).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // U is subsampled by 2
                width: self.frame_info.width as u32 / 2,
                height: self.frame_info.height as u32 / 2,
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tex_v,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            yuv.get_v_plane(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(self.frame_info.width as u32 / 2).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                // V is not subsampled
                width: self.frame_info.width as u32 / 2,
                height: self.frame_info.height as u32 / 2,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn bind_group(&self) -> &YuvTextureBindGroup {
        &self.bind_group
    }
}
