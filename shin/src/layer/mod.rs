use crate::interpolator::Interpolator;
use enum_map::EnumMap;
use shin_core::vm::command::layer::LayerProperty;

pub struct LayerProperties {
    properies: EnumMap<LayerProperty, Interpolator>,
}

impl LayerProperties {
    //
}
