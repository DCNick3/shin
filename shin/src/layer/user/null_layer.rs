use std::fmt::Debug;

use shin_render::{render_pass::RenderPass, PassKind};

use crate::{
    layer::{properties::LayerProperties, render_params::TransformParams, DrawableLayer, Layer},
    update::{AdvUpdatable, AdvUpdateContext},
};

#[derive(Clone)]
pub struct NullLayer {
    props: LayerProperties,
}

impl NullLayer {
    pub fn new() -> Self {
        Self {
            props: LayerProperties::new(),
        }
    }
}

impl AdvUpdatable for NullLayer {
    fn update(&mut self, _ctx: &AdvUpdateContext) {}
}

impl Debug for NullLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NullLayer").finish()
    }
}

impl Layer for NullLayer {
    fn render(
        &self,
        _pass: &mut RenderPass,
        _transform: &TransformParams,
        _stencil_ref: u8,
        _pass_kind: PassKind,
    ) {
    }
}

impl DrawableLayer for NullLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
