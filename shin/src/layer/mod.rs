mod bustup_layer;
mod layer_group;
mod message_layer;
mod movie_layer;
mod new_drawable_layer;
mod null_layer;
mod page_layer;
mod picture_layer;
mod properties;
mod render_params;
mod root_layer_group;
mod screen_layer;
mod tile_layer;
mod wobbler;

use std::f32::consts::PI;

pub use bustup_layer::BustupLayer;
use derivative::Derivative;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use enum_map::{enum_map, EnumMap};
use glam::{vec3, Mat4};
pub use layer_group::LayerGroup;
pub use message_layer::MessageLayer;
pub use movie_layer::MovieLayer;
pub use new_drawable_layer::{DrawableLayer, NewDrawableLayer, NewDrawableLayerWrapper};
pub use null_layer::NullLayer;
pub use page_layer::PageLayer;
pub use picture_layer::PictureLayer;
pub use properties::{LayerProperties, LayerPropertiesSnapshot};
pub use root_layer_group::RootLayerGroup;
pub use screen_layer::ScreenLayer;
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::{
        info::{BustupInfoItem, MovieInfoItem, PictureInfoItem},
        instruction_elements::UntypedNumberArray,
        Scenario,
    },
    time::{Ticks, Tweener},
    vm::command::types::{LayerProperty, LayerType},
};
pub use tile_layer::TileLayer;
use tracing::{debug, warn};

use crate::{
    asset::{bustup::Bustup, movie::Movie, picture::Picture, system::AssetServer},
    layer::wobbler::Wobbler,
    update::{Updatable, UpdateContext},
};

#[enum_dispatch]
pub trait Layer:
// Renderable +
Updatable {
    fn properties(&self) -> &LayerProperties;
    fn properties_mut(&mut self) -> &mut LayerProperties;
}

#[enum_dispatch(Layer, Updatable)]
#[derive(Derivative)]
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
                let (tile_color, offset_x, offset_y, width, height, ..) = params;
                TileLayer::new(tile_color, offset_x, offset_y, width, height).into()
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

#[derive(From)]
pub enum AnyLayer<'a> {
    UserLayer(&'a UserLayer),
    RootLayerGroup(&'a RootLayerGroup),
    ScreenLayer(&'a ScreenLayer),
    PageLayer(&'a PageLayer),
    LayerGroup(&'a LayerGroup),
}

impl<'a> AnyLayer<'a> {
    #[allow(unused)]
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::RootLayerGroup(layer) => layer.properties(),
            Self::ScreenLayer(layer) => layer.properties(),
            Self::PageLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }
}

#[derive(From)]
pub enum AnyLayerMut<'a> {
    UserLayer(&'a mut UserLayer),
    RootLayerGroup(&'a mut RootLayerGroup),
    ScreenLayer(&'a mut ScreenLayer),
    PageLayer(&'a mut PageLayer),
    LayerGroup(&'a mut LayerGroup),
}

impl<'a> AnyLayerMut<'a> {
    #[allow(unused)]
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::RootLayerGroup(layer) => layer.properties(),
            Self::ScreenLayer(layer) => layer.properties(),
            Self::PageLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }

    pub fn properties_mut(&mut self) -> &mut LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties_mut(),
            Self::RootLayerGroup(layer) => layer.properties_mut(),
            Self::ScreenLayer(layer) => layer.properties_mut(),
            Self::PageLayer(layer) => layer.properties_mut(),
            Self::LayerGroup(layer) => layer.properties_mut(),
        }
    }
}
