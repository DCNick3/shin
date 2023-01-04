mod bustup_layer;
mod layer_group;
mod message_layer;
mod null_layer;
mod picture_layer;
mod root_layer_group;

use cgmath::{Matrix4, SquareMatrix, Vector3};
use derive_more::From;
use enum_dispatch::enum_dispatch;
use enum_map::{enum_map, EnumMap};
use strum::IntoStaticStr;
use tracing::{debug, warn};

pub use bustup_layer::BustupLayer;
pub use layer_group::LayerGroup;
pub use message_layer::{MessageLayer, MessageboxTextures};
pub use null_layer::NullLayer;
pub use picture_layer::PictureLayer;
pub use root_layer_group::RootLayerGroup;

use crate::asset::bustup::Bustup;
use crate::asset::picture::Picture;
use crate::asset::AnyAssetServer;
use crate::interpolator::{Easing, Interpolator};
use crate::render::{GpuCommonResources, Renderable};
use crate::update::{Updatable, UpdateContext};
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::layer::{LayerProperty, LayerType};
use shin_core::vm::command::time::Ticks;

fn initial_values() -> EnumMap<LayerProperty, i32> {
    enum_map! {
        v => v.initial_value()
    }
}

pub struct LayerProperties {
    properties: EnumMap<LayerProperty, Interpolator>,
}

impl LayerProperties {
    pub fn new() -> Self {
        Self {
            properties: initial_values().map(|_, v| Interpolator::new(v as f32)),
        }
    }

    pub fn get_property(&self, property: LayerProperty) -> f32 {
        self.properties[property].value()
    }

    pub fn set_property(
        &mut self,
        property: LayerProperty,
        value: f32,
        time: Ticks,
        easing: Easing,
    ) {
        self.properties[property].enqueue(value, time, easing);
    }

    pub fn init(&mut self) {
        for (prop, val) in initial_values() {
            self.properties[prop].enqueue_force(val as f32);
        }
    }

    pub fn compute_transform(&self, base_transform: Matrix4<f32>) -> Matrix4<f32> {
        macro_rules! get {
            (Zero) => {
                0.0
            };
            ($property:ident) => {
                self.get_property(LayerProperty::$property)
            };
            ($x_property:ident, $y_property:ident, $z_property:ident) => {
                Vector3::new(get!($x_property), get!($y_property), get!($z_property))
            };
        }

        // TODO: actually use all the properties

        let transforms = [
            Matrix4::from_translation(get!(TranslateX, TranslateY, Zero)),
            Matrix4::from_angle_z(cgmath::Deg(get!(Rotation))), // TODO: handle rotation origin
            base_transform,
        ];

        transforms
            .into_iter()
            .fold(Matrix4::identity(), |acc, t| t * acc)
    }
}

impl Updatable for LayerProperties {
    fn update(&mut self, ctx: &UpdateContext) {
        for property in self.properties.values_mut() {
            property.update(ctx);
        }
    }
}

/// Stores only target property values.
/// Used to implement save/load (to quickly restore the state of the scene).
#[derive(Debug, Copy, Clone)]
pub struct LayerPropertiesSnapshot {
    // The game can actually only set integer values
    // hence the the use of i32 instead of f32
    properties: EnumMap<LayerProperty, i32>,
}

impl LayerPropertiesSnapshot {
    pub fn new() -> Self {
        Self {
            properties: initial_values(),
        }
    }

    pub fn init(&mut self) {
        self.properties = initial_values();
    }

    #[allow(unused)]
    pub fn get_property(&self, property: LayerProperty) -> i32 {
        self.properties[property]
    }

    pub fn set_property(&mut self, property: LayerProperty, value: i32) {
        self.properties[property] = value;
    }
}

#[enum_dispatch]
pub trait Layer: Renderable + Updatable {
    fn properties(&self) -> &LayerProperties;
    fn properties_mut(&mut self) -> &mut LayerProperties;
}

#[enum_dispatch(Layer, Renderable, Updatable)]
#[derive(IntoStaticStr)]
pub enum UserLayer {
    NullLayer,
    PictureLayer,
    BustupLayer,
}

impl UserLayer {
    pub async fn load(
        resources: &GpuCommonResources,
        asset_server: &AnyAssetServer,
        scenario: &Scenario,
        layer_ty: LayerType,
        params: [i32; 8],
    ) -> Self {
        // TODO: this API is not ideal, as we are blocking the main thread for layer loading
        // ideally we want to mimic the API of LayerLoader in the original game
        match layer_ty {
            LayerType::Null => NullLayer::new().into(),
            LayerType::Tile => {
                warn!("Loading NullLayer instead of TileLayer");
                NullLayer::new().into()
            }
            LayerType::Picture => {
                let [pic_id, _, _, _, _, _, _, _] = params;
                let (pic_name, v1) = scenario.get_picture_data(pic_id);
                debug!("Load picture: {} -> {} {}", pic_id, pic_name, v1);
                let pic_path = format!("/picture/{}.pic", pic_name.to_ascii_lowercase());
                let pic = asset_server
                    .load::<Picture>(&pic_path)
                    .await
                    .expect("Failed to load picture");
                PictureLayer::new(resources, pic).into()
            }
            LayerType::Bustup => {
                let [bup_id, _, _, _, _, _, _, _] = params;
                let (bup_name, bup_emotion, v1) = scenario.get_bustup_data(bup_id);
                debug!(
                    "Load bustup: {} -> {} {} {}",
                    bup_id, bup_name, bup_emotion, v1
                );
                let bup_path = format!("/bustup/{}.bup", bup_name.to_ascii_lowercase());
                let bup = asset_server
                    .load::<Bustup>(&bup_path)
                    .await
                    .expect("Failed to load bustup");

                BustupLayer::new(resources, bup, bup_emotion).into()
            }
            _ => {
                todo!("Layer type not implemented: {:?}", layer_ty);
            }
        }
    }
}

#[derive(From)]
pub enum AnyLayer<'a> {
    UserLayer(&'a UserLayer),
    RootLayerGroup(&'a RootLayerGroup),
    MessageLayer(&'a MessageLayer),
    LayerGroup(&'a LayerGroup),
}

impl<'a> AnyLayer<'a> {
    #[allow(unused)]
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::RootLayerGroup(layer) => layer.properties(),
            Self::MessageLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }
}

#[derive(From)]
pub enum AnyLayerMut<'a> {
    UserLayer(&'a mut UserLayer),
    RootLayerGroup(&'a mut RootLayerGroup),
    MessageLayer(&'a mut MessageLayer),
    LayerGroup(&'a mut LayerGroup),
}

impl<'a> AnyLayerMut<'a> {
    #[allow(unused)]
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::RootLayerGroup(layer) => layer.properties(),
            Self::MessageLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }

    pub fn properties_mut(&mut self) -> &mut LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties_mut(),
            Self::RootLayerGroup(layer) => layer.properties_mut(),
            Self::MessageLayer(layer) => layer.properties_mut(),
            Self::LayerGroup(layer) => layer.properties_mut(),
        }
    }
}
