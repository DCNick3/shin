use std::sync::Arc;

use shin_render_shader_types::texture::DepthStencilTarget;

use crate::{
    resize::{CanvasSize, ResizeHandle},
    resizeable_texture::ResizeableTexture,
    DEPTH_STENCIL_FORMAT,
};

#[derive(Debug)]
pub struct DepthStencil {
    // TODO: how to properly share this texture with multiple owners? Resizing won't be very nice...
    // maybe provide it externally?
    inner_texture: ResizeableTexture<CanvasSize>,
    #[expect(unused)] // for debugging or idk, let it stay for now
    label: String,
}

impl DepthStencil {
    pub fn new(
        device: Arc<wgpu::Device>,
        resize_handle: ResizeHandle<CanvasSize>,
        label: Option<String>,
    ) -> Self {
        let label = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let inner_texture = ResizeableTexture::new(
            device.clone(),
            Some(label.clone()),
            DEPTH_STENCIL_FORMAT,
            resize_handle,
        );

        Self {
            inner_texture,
            label,
        }
    }

    pub fn get_target_view(&mut self) -> DepthStencilTarget {
        DepthStencilTarget {
            view: self.inner_texture.resize_and_get_view(),
        }
    }
}
