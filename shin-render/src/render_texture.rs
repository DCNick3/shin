use shin_render_shader_types::{
    RenderClone, RenderCloneCtx,
    texture::{TextureSampler, TextureSource, TextureTarget, TextureTargetKind},
};

use crate::{
    TEXTURE_FORMAT,
    resize::{CanvasSize, ResizeHandle},
    resizeable_texture::ResizeableTexture,
};

const TEXTURE_USAGES: wgpu::TextureUsages = {
    // not using `|` here because rust doesn't have const fn in traits
    // see https://github.com/bitflags/bitflags/issues/399
    let mut result = wgpu::TextureUsages::empty();

    // for rendering into
    result = result.union(wgpu::TextureUsages::RENDER_ATTACHMENT);
    // for binding as a texture
    result = result.union(wgpu::TextureUsages::TEXTURE_BINDING);
    // for cloning
    result = result.union(wgpu::TextureUsages::COPY_SRC);
    result = result.union(wgpu::TextureUsages::COPY_DST);

    result
};

#[derive(Debug)]
pub struct RenderTexture {
    inner_texture: ResizeableTexture<CanvasSize>,
    sampler: TextureSampler,
    label: String,
}

impl RenderTexture {
    pub fn new(
        device: wgpu::Device,
        resize_handle: ResizeHandle<CanvasSize>,
        label: String,
    ) -> Self {
        let sampler = TextureSampler::Linear;

        let inner_texture = ResizeableTexture::new(
            device,
            label.clone(),
            TEXTURE_FORMAT,
            TEXTURE_USAGES,
            resize_handle,
        );

        Self {
            inner_texture,
            sampler,
            label,
        }
    }

    pub fn as_texture_source(&self) -> TextureSource {
        TextureSource {
            view: self.inner_texture.get_view(),
            sampler: self.sampler,
        }
    }

    pub fn as_texture_target(&mut self) -> TextureTarget {
        TextureTarget {
            kind: TextureTargetKind::RenderTexture,
            view: self.inner_texture.resize_and_get_view(),
        }
    }
}

impl RenderClone for RenderTexture {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        let resize_handle = self.inner_texture.get_resize_handle();

        let size = resize_handle.get_without_update().into();

        let new_texture = ResizeableTexture::new_with_size(
            ctx.device.clone(),
            // TODO: having two object with the same name can be confusing
            self.label.clone(),
            TEXTURE_FORMAT,
            TEXTURE_USAGES,
            size,
            resize_handle,
        );

        ctx.encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self.inner_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: new_texture.get_texture(),
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

        Self {
            inner_texture: new_texture,
            sampler: self.sampler,
            label: self.label.clone(),
        }
    }
}
