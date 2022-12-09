mod layer_group;
mod null_layer;
mod picture_layer;

use cgmath::{Matrix4, SquareMatrix, Vector3};
use enum_dispatch::enum_dispatch;

pub use layer_group::LayerGroup;
pub use null_layer::NullLayer;
pub use picture_layer::PictureLayer;

use crate::asset;
use crate::asset::picture::GpuPicture;
use crate::game_data::GameData;
use crate::interpolator::{Easing, Interpolator};
use crate::render::{GpuCommonResources, Renderable};
use crate::update::{Ticks, Updatable, UpdateContext};
use enum_map::{Enum, EnumMap};
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::layer::{LayerProperty, LayerType};
use tracing::{debug, warn};

fn initial_values() -> EnumMap<LayerProperty, i32> {
    EnumMap::from_array(
        (0..LayerProperty::COUNT)
            .map(|i| <LayerProperty as Enum>::from_usize(i).initial_value())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    )
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
pub enum UserLayer {
    NullLayer,
    PictureLayer,
}

impl UserLayer {
    pub fn load(
        resources: &GpuCommonResources,
        game_data: &GameData,
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
                let pic_data = game_data.read_file(&pic_path);
                let pic = asset::picture::load_picture(&pic_data).expect("Parsing picture");
                let pic = GpuPicture::load(resources, pic);
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
                todo!()
                // warn!("Loading NullLayer instead of BustupLayer");
                // NullLayer::new().into()
            }
            _ => {
                todo!("Layer type not implemented: {:?}", layer_ty);
            }
        }
    }
}

pub enum AnyLayerMut<'a> {
    UserLayer(&'a mut UserLayer),
    LayerGroup(&'a mut LayerGroup),
}

impl<'a> AnyLayerMut<'a> {
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }

    pub fn properties_mut(&mut self) -> &mut LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties_mut(),
            Self::LayerGroup(layer) => layer.properties_mut(),
        }
    }
}

impl<'a> From<&'a mut UserLayer> for AnyLayerMut<'a> {
    fn from(layer: &'a mut UserLayer) -> Self {
        Self::UserLayer(layer)
    }
}

impl<'a> From<&'a mut LayerGroup> for AnyLayerMut<'a> {
    fn from(layer: &'a mut LayerGroup) -> Self {
        Self::LayerGroup(layer)
    }
}
