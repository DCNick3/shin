use derivative::Derivative;
use enum_dispatch::enum_dispatch;
use from_variants::FromVariants;
use glam::vec4;
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::{
        info::{BustupInfoItem, MovieInfoItem, PictureInfoItem},
        instruction_elements::UntypedNumberArray,
        Scenario,
    },
    vm::command::types::LayerType,
};
use shin_render::{render_pass::RenderPass, shaders::types::vertices::FloatColor4, PassKind};
use tracing::{debug, warn};

use crate::{
    asset::{bustup::Bustup, movie::Movie, picture::Picture, system::AssetServer},
    layer::{
        render_params::TransformParams, DrawableLayer, Layer, LayerProperties, PreRenderContext,
    },
    update::{Updatable, UpdateContext},
};

mod bustup_layer;
mod movie_layer;
mod null_layer;
mod picture_layer;
mod tile_layer;

pub use self::{
    bustup_layer::BustupLayer, movie_layer::MovieLayer, null_layer::NullLayer,
    picture_layer::PictureLayer, tile_layer::TileLayer,
};

#[derive(Derivative, Clone, FromVariants)]
#[derivative(Debug)]
pub enum UserLayer {
    #[derivative(Debug = "transparent")]
    NullLayer(NullLayer),
    #[derivative(Debug = "transparent")]
    PictureLayer(PictureLayer),
    #[derivative(Debug = "transparent")]
    BustupLayer(BustupLayer),
    #[derivative(Debug = "transparent")]
    TileLayer(TileLayer),
    #[derivative(Debug = "transparent")]
    MovieLayer(MovieLayer),
}

impl UserLayer {
    pub async fn load(
        device: &wgpu::Device,
        // resources: &GpuCommonResources,
        asset_server: &AssetServer,
        audio_manager: &AudioManager,
        scenario: &Scenario,
        layer_ty: LayerType,
        params: UntypedNumberArray,
    ) -> Self {
        // TODO: this API is not ideal, as we are blocking the main thread for layer loading
        // ideally we want to mimic the API of LayerLoader in the original game
        match layer_ty {
            LayerType::Null => NullLayer::new().into(),
            LayerType::Tile => {
                let (color, offset_x, offset_y, width, height, ..) = params;
                let color = FloatColor4::from_4bpp_property(color);
                let rect = vec4(
                    offset_x as f32,
                    offset_y as f32,
                    width as f32,
                    height as f32,
                );

                TileLayer::new(color, rect).into()
            }
            LayerType::Picture => {
                let (pic_id, ..) = params;
                let pic_info @ PictureInfoItem { name, linked_cg_id } =
                    scenario.info_tables().picture_info(pic_id);
                debug!("Load picture: {} -> {} {}", pic_id, name, linked_cg_id);
                let pic = asset_server
                    .load::<Picture, _>(pic_info.path())
                    .await
                    .expect("Failed to load picture");
                PictureLayer::new(pic, Some(name.to_string())).into()
            }
            LayerType::Bustup => {
                let (bup_id, ..) = params;
                let bup_info @ BustupInfoItem {
                    name,
                    emotion,
                    lipsync_character_id,
                } = scenario.info_tables().bustup_info(bup_id);
                debug!(
                    "Load bustup: {} -> {} {} {}",
                    bup_id, name, emotion, lipsync_character_id
                );
                let bup = asset_server
                    .load::<Bustup, _>(bup_info.path())
                    .await
                    .expect("Failed to load bustup");

                BustupLayer::new(bup, Some(name.to_string()), emotion.as_str()).into()
            }
            LayerType::Movie => {
                let (movie_id, _volume, _flags, ..) = params;
                let movie_info @ MovieInfoItem {
                    name,
                    linked_picture_id,
                    volume_source,
                    transparency,
                    linked_bgm_id,
                } = scenario.info_tables().movie_info(movie_id);
                debug!(
                    "Load movie: {movie_id} -> {name} {linked_picture_id} {volume_source:?} {transparency:?} {linked_bgm_id}"
                );
                let movie = asset_server
                    .load::<Movie, _>(movie_info.path())
                    .await
                    .expect("Failed to load movie");

                MovieLayer::new(device, audio_manager, movie, Some(name.to_string())).into()
            }
            LayerType::Rain => {
                let (_always_zero, _min_distance, _max_distance, ..) = params;

                warn!("Loading NullLayer instead of RainLayer");
                NullLayer::new().into()
            }
            _ => {
                todo!("Layer type not implemented: {:?}", layer_ty);
            }
        }
    }
}

impl Updatable for UserLayer {
    fn update(&mut self, context: &UpdateContext) {
        match self {
            Self::NullLayer(layer) => layer.update(context),
            Self::PictureLayer(layer) => layer.update(context),
            Self::BustupLayer(layer) => layer.update(context),
            Self::TileLayer(layer) => layer.update(context),
            Self::MovieLayer(layer) => layer.update(context),
        }
    }
}

impl DrawableLayer for UserLayer {
    fn properties(&self) -> &LayerProperties {
        match self {
            Self::NullLayer(layer) => layer.properties(),
            Self::PictureLayer(layer) => layer.properties(),
            Self::BustupLayer(layer) => layer.properties(),
            Self::TileLayer(layer) => layer.properties(),
            Self::MovieLayer(layer) => layer.properties(),
        }
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        match self {
            Self::NullLayer(layer) => layer.properties_mut(),
            Self::PictureLayer(layer) => layer.properties_mut(),
            Self::BustupLayer(layer) => layer.properties_mut(),
            Self::TileLayer(layer) => layer.properties_mut(),
            Self::MovieLayer(layer) => layer.properties_mut(),
        }
    }
}

impl Layer for UserLayer {
    fn get_stencil_bump(&self) -> u8 {
        match self {
            Self::NullLayer(layer) => layer.get_stencil_bump(),
            Self::PictureLayer(layer) => layer.get_stencil_bump(),
            Self::BustupLayer(layer) => layer.get_stencil_bump(),
            Self::TileLayer(layer) => layer.get_stencil_bump(),
            Self::MovieLayer(layer) => layer.get_stencil_bump(),
        }
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        match self {
            Self::NullLayer(layer) => layer.pre_render(context, transform),
            Self::PictureLayer(layer) => layer.pre_render(context, transform),
            Self::BustupLayer(layer) => layer.pre_render(context, transform),
            Self::TileLayer(layer) => layer.pre_render(context, transform),
            Self::MovieLayer(layer) => layer.pre_render(context, transform),
        }
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        match self {
            Self::NullLayer(layer) => layer.render(pass, transform, stencil_ref, pass_kind),
            Self::PictureLayer(layer) => layer.render(pass, transform, stencil_ref, pass_kind),
            Self::BustupLayer(layer) => layer.render(pass, transform, stencil_ref, pass_kind),
            Self::TileLayer(layer) => layer.render(pass, transform, stencil_ref, pass_kind),
            Self::MovieLayer(layer) => layer.render(pass, transform, stencil_ref, pass_kind),
        }
    }
}
