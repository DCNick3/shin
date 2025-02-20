mod default;
mod mask;
mod timed;

use from_variants::FromVariants;
use shin_core::{
    format::scenario::{
        info::MaskId,
        instruction_elements::{lower_number_array, TypedNumberArray, UntypedNumberArray},
        Scenario,
    },
    time::Ticks,
    vm::command::types::{MaskFlags, MaskParam, WiperType},
};
use shin_derive::RenderClone;
use shin_render::{
    render_pass::RenderPass, shaders::types::texture::TextureSource, RenderRequestBuilder,
};

pub use self::default::DefaultWiper;
use crate::{
    asset::{mask::MaskTexture, system::AssetServer},
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::mask::MaskWiper,
};

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

#[derive(Debug, RenderClone, FromVariants)]
pub enum AnyWiper {
    Default(DefaultWiper),
    Mask(MaskWiper),
}

impl AnyWiper {
    pub async fn load(
        asset_server: &AssetServer,
        scenario: &Scenario,
        ty: WiperType,
        duration: Ticks,
        params: UntypedNumberArray,
    ) -> AnyWiper {
        match ty {
            WiperType::Default => AnyWiper::Default(DefaultWiper::new(duration)),
            WiperType::Mask => {
                let (mask_id, param2, flags, _, _, _, _, _): TypedNumberArray<
                    MaskId,
                    MaskParam,
                    MaskFlags,
                > = lower_number_array(params);

                // NB: original engine falls back to DefaultWiper if `mask_id` points to a non-existing mask
                // we will just panic
                let mask_info = scenario.info_tables().mask_info(mask_id);
                let mask_path = mask_info.path();

                let mask: std::sync::Arc<MaskTexture> = asset_server
                    .load(&mask_path)
                    .await
                    .expect("Failed to load mask");

                AnyWiper::Mask(MaskWiper::new(duration, mask, param2, flags))
            }
            ty => {
                todo!("unimplemented wiper {:?} {:?} {:?}", ty, duration, params)
            }
        }
    }
}

impl AdvUpdatable for AnyWiper {
    fn update(&mut self, context: &AdvUpdateContext) {
        match self {
            AnyWiper::Default(wiper) => wiper.update(context),
            AnyWiper::Mask(wiper) => wiper.update(context),
        }
    }
}

impl Wiper for AnyWiper {
    fn is_running(&self) -> bool {
        match self {
            AnyWiper::Default(wiper) => wiper.is_running(),
            AnyWiper::Mask(wiper) => wiper.is_running(),
        }
    }

    fn fast_forward(&mut self) {
        match self {
            AnyWiper::Default(wiper) => wiper.fast_forward(),
            AnyWiper::Mask(wiper) => wiper.fast_forward(),
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
            AnyWiper::Mask(wiper) => {
                wiper.render(pass, render_request_builder, from_texture, to_texture)
            }
        }
    }
}
