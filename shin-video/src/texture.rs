use dpi::PhysicalSize;
use shin_render::{
    gpu_texture::{GpuTexture, TextureKind},
    shaders::types::texture::TextureSource,
};

use crate::h264_decoder::Nv12Frame;

// we could utilize `wgpu::Features::TEXTURE_FORMAT_NV12` to have some native YUV textures, but it's not supported on all platforms
// so we are going to store Y and UV planes separately
pub struct VideoFrameTexture {
    tex_y: GpuTexture,
    tex_uv: GpuTexture,
    size: PhysicalSize<u32>,
}

impl VideoFrameTexture {
    pub fn new(device: &wgpu::Device, size: PhysicalSize<u32>) -> Self {
        let tex_y = GpuTexture::new_empty(
            device,
            Some("VideoFrameTexture Y"),
            (size.width, size.height).into(),
            wgpu::TextureFormat::R8Unorm,
            TextureKind::Updatable,
        );
        let tex_uv = GpuTexture::new_empty(
            device,
            Some("VideoFrameTexture UV"),
            (size.width / 2, size.height / 2).into(),
            wgpu::TextureFormat::Rg8Unorm,
            TextureKind::Updatable,
        );

        Self {
            tex_y,
            tex_uv,
            size,
        }
    }

    pub fn write_data_nv12(&self, queue: &wgpu::Queue, nv12: &Nv12Frame) {
        // TODO: maybe support resizing the texture if the frame size changes?
        assert_eq!(self.size, nv12.size);

        let size = self.size;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.tex_y.wgpu_texture(),
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &nv12.y_plane,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size.width),
                rows_per_image: Some(size.height),
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.tex_uv.wgpu_texture(),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            &nv12.uv_plane,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size.width),
                rows_per_image: Some(size.height / 2),
            },
            wgpu::Extent3d {
                width: size.width / 2,
                height: size.height / 2,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_y_source(&self) -> TextureSource {
        self.tex_y.as_source()
    }

    pub fn get_uv_source(&self) -> TextureSource {
        self.tex_uv.as_source()
    }
}
