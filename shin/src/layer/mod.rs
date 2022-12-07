mod picture_layer;

pub use picture_layer::PictureLayer;

use crate::interpolator::{Easing, Interpolator};
use crate::render::Renderable;
use crate::update::{Ticks, Updatable, UpdateContext};
use enum_map::{Enum, EnumMap};
use shin_core::vm::command::layer::LayerProperty;

pub struct LayerProperties {
    properties: EnumMap<LayerProperty, Interpolator>,
}

impl LayerProperties {
    pub fn new() -> Self {
        Self {
            properties: EnumMap::from_array(
                (0..LayerProperty::COUNT)
                    .map(|i| {
                        Interpolator::new(<LayerProperty as Enum>::from_usize(i).initial_value())
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
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
}

impl Updatable for LayerProperties {
    fn update(&mut self, ctx: &UpdateContext) {
        for property in self.properties.values_mut() {
            property.update(ctx);
        }
    }
}

pub trait Layer: Renderable + Updatable {
    fn properties(&self) -> &LayerProperties;
    fn properties_mut(&mut self) -> &mut LayerProperties;
}
