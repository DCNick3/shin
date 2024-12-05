use derivative::Derivative;
use enum_dispatch::enum_dispatch;
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
use shin_render::shaders::types::vertices::FloatColor4;
use tracing::{debug, warn};

use crate::asset::{bustup::Bustup, movie::Movie, picture::Picture, system::AssetServer};

mod bustup_layer;
mod movie_layer;
mod null_layer;
mod picture_layer;
mod tile_layer;

pub use self::{
    bustup_layer::BustupLayer, movie_layer::MovieLayer, null_layer::NullLayer,
    picture_layer::PictureLayer, tile_layer::TileLayer,
};

#[enum_dispatch(DrawableLayer, Layer, Updatable)]
#[derive(Derivative, Clone)]
#[derivative(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum UserLayer {
    #[derivative(Debug = "transparent")]
    NullLayer,
    #[derivative(Debug = "transparent")]
    PictureLayer,
    #[derivative(Debug = "transparent")]
    BustupLayer,
    #[derivative(Debug = "transparent")]
    TileLayer,
    #[derivative(Debug = "transparent")]
    MovieLayer,
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

// impl Renderable for UserLayer {
//     fn render<'enc>(
//         &'enc self,
//         resources: &'enc GpuCommonResources,
//         render_pass: &mut wgpu::RenderPass<'enc>,
//         transform: Mat4,
//         projection: Mat4,
//     ) {
//         match self {
//             UserLayer::NullLayer(l) => l.render(resources, render_pass, transform, projection),
//             UserLayer::PictureLayer(l) => l.render(resources, render_pass, transform, projection),
//             UserLayer::BustupLayer(l) => l.render(resources, render_pass, transform, projection),
//             UserLayer::TileLayer(l) => l.render(resources, render_pass, transform, projection),
//             UserLayer::MovieLayer(l) => l.render(resources, render_pass, transform, projection),
//         }
//     }
//
//     fn resize(&mut self, resources: &GpuCommonResources) {
//         match self {
//             UserLayer::NullLayer(l) => l.resize(resources),
//             UserLayer::PictureLayer(l) => l.resize(resources),
//             UserLayer::BustupLayer(l) => l.resize(resources),
//             UserLayer::TileLayer(l) => l.resize(resources),
//             UserLayer::MovieLayer(l) => l.resize(resources),
//         }
//     }
// }
