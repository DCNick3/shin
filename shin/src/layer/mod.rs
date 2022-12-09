mod layer_group;
mod picture_layer;

use cgmath::{Matrix4, SquareMatrix, Vector3};
use enum_dispatch::enum_dispatch;
pub use layer_group::LayerGroup;
pub use picture_layer::PictureLayer;

use crate::interpolator::{Easing, Interpolator};
use crate::render::Renderable;
use crate::update::{Ticks, Updatable, UpdateContext};
use enum_map::{Enum, EnumMap};
use shin_core::vm::command::layer::LayerProperty;

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
    PictureLayer,
}

impl UserLayer {}
