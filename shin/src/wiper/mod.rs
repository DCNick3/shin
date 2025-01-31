mod default;
mod timed;

use from_variants::FromVariants;
use shin_derive::RenderClone;
use shin_render::{
    render_pass::RenderPass, shaders::types::texture::TextureSource, RenderRequestBuilder,
};

pub use self::default::DefaultWiper;
use crate::update::{AdvUpdatable, AdvUpdateContext};

pub trait Wiper: AdvUpdatable {
    fn is_running(&self) -> bool;
    fn fast_forward(&mut self);
    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        texture_target: TextureSource,
        texture_source: TextureSource,
    );
}

#[derive(RenderClone, FromVariants)]
pub enum AnyWiper {
    Default(DefaultWiper),
}

impl AdvUpdatable for AnyWiper {
    fn update(&mut self, context: &AdvUpdateContext) {
        match self {
            AnyWiper::Default(wiper) => wiper.update(context),
        }
    }
}

impl Wiper for AnyWiper {
    fn is_running(&self) -> bool {
        match self {
            AnyWiper::Default(wiper) => wiper.is_running(),
        }
    }

    fn fast_forward(&mut self) {
        match self {
            AnyWiper::Default(wiper) => wiper.fast_forward(),
        }
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        render_request_builder: RenderRequestBuilder,
        from_texture: TextureSource,
        to_texture: TextureSource,
    ) {
        match self {
            AnyWiper::Default(wiper) => {
                wiper.render(pass, render_request_builder, from_texture, to_texture)
            }
        }
    }
}
