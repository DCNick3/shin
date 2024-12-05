mod layer_group;
mod message_layer;
mod new_drawable_layer;
mod page_layer;
mod properties;
pub mod render_params;
mod root_layer_group;
mod screen_layer;
pub mod user;
mod wobbler;

use derive_more::From;
use enum_dispatch::enum_dispatch;
pub use layer_group::LayerGroup;
pub use message_layer::MessageLayer;
pub use new_drawable_layer::{NewDrawableLayer, NewDrawableLayerWrapper};
pub use page_layer::PageLayer;
pub use properties::{LayerProperties, LayerPropertiesSnapshot};
pub use root_layer_group::RootLayerGroup;
pub use screen_layer::ScreenLayer;
use shin_render::{render_pass::RenderPass, PassKind};

// need those imports for enum_dispatch to work (eww)
use self::user::{BustupLayer, MovieLayer, NullLayer, PictureLayer, TileLayer};
use crate::{
    layer::{render_params::TransformParams, user::UserLayer},
    update::Updatable,
};

#[enum_dispatch]
pub trait Layer: Clone + Updatable {
    // fn fast_forward(&mut self);
    fn get_stencil_bump(&self) -> u32 {
        1
    }
    fn pre_render(&mut self) {}
    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    );
}

#[enum_dispatch]
pub trait DrawableLayer: Layer {
    // fn init(&mut self);
    // fn set_properties(&mut self, properties: LayerProperties);
    fn properties(&self) -> &LayerProperties;
    fn properties_mut(&mut self) -> &mut LayerProperties;
}

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
