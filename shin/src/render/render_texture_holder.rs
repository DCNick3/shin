use std::borrow::Cow;

use shin_render::{
    render_texture::RenderTexture,
    shaders::types::{RenderClone, RenderCloneCtx},
};

use crate::render::PreRenderContext;

#[derive(Debug)]
pub struct RenderTextureHolder {
    label: Cow<'static, str>,
    render_texture: Option<RenderTexture>,
    render_texture_alloc_counter: u32,
}

impl RenderTextureHolder {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            render_texture: None,
            render_texture_alloc_counter: 0,
        }
    }

    pub fn get_or_init(&mut self, ctx: &PreRenderContext) -> &mut RenderTexture {
        self.render_texture.get_or_insert_with(|| {
            let res = ctx.new_render_texture(format!(
                "{} #{}",
                self.label, self.render_texture_alloc_counter
            ));
            self.render_texture_alloc_counter += 1;
            res
        })
    }

    #[expect(unused)] // this provides a more comprehensive API
    pub fn take(&mut self) -> Option<RenderTexture> {
        self.render_texture.take()
    }

    pub fn clear(&mut self) {
        self.render_texture = None;
    }

    pub fn get(&self) -> Option<&RenderTexture> {
        self.render_texture.as_ref()
    }

    #[expect(unused)] // this provides a more comprehensive API
    pub fn get_mut(&mut self) -> Option<&mut RenderTexture> {
        self.render_texture.as_mut()
    }

    #[expect(unused)] // this provides a more comprehensive API
    pub fn as_inner(&self) -> &Option<RenderTexture> {
        &self.render_texture
    }

    pub fn as_inner_mut(&mut self) -> &mut Option<RenderTexture> {
        &mut self.render_texture
    }
}

impl RenderClone for RenderTextureHolder {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        Self {
            label: self.label.clone(),
            render_texture: self.render_texture.render_clone(ctx),
            render_texture_alloc_counter: 0,
        }
    }
}
