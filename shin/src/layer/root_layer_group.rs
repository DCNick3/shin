use std::sync::Arc;

use from_variants::FromVariants;
use glam::Mat4;
use shin_core::{format::scenario::Scenario, vm::command::types::LayerbankId};
use shin_derive::RenderClone;
use shin_render::{render_pass::RenderPass, PassKind};

use crate::{
    adv::assets::AdvFonts,
    audio::VoicePlayer,
    layer::{
        message_layer::{MessageLayer, MessageboxTextures},
        properties::LayerProperties,
        render_layer_without_bg,
        render_params::TransformParams,
        screen_layer::ScreenLayer,
        DrawableLayer, Layer, LayerGroup, PreRenderContext,
    },
    update::{AdvUpdatable, AdvUpdateContext, Updatable, UpdateContext},
};

const OVERLAY_LAYERBANK: LayerbankId = LayerbankId::new_unchecked(0);
const MESSAGE_LAYERBANK: LayerbankId = LayerbankId::new_unchecked(1);
const SCREEN_LAYERBANK: LayerbankId = LayerbankId::new_unchecked(2);

#[derive(RenderClone, FromVariants)]
enum RootLayer {
    // Overlay(OverlayLayer),
    Message(MessageLayer),
    Screen(#[render_clone(needs_render)] ScreenLayer),
}

impl AdvUpdatable for RootLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
        match self {
            // RootLayer::Overlay(overlay) => overlay.update(context),
            RootLayer::Message(message) => message.update(context),
            RootLayer::Screen(screen) => screen.update(context),
        }
    }
}

impl Layer for RootLayer {
    fn get_stencil_bump(&self) -> u8 {
        match self {
            // RootLayer::Overlay(overlay) => overlay.get_stencil_bump(),
            RootLayer::Message(message) => message.get_stencil_bump(),
            RootLayer::Screen(screen) => screen.get_stencil_bump(),
        }
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        match self {
            // RootLayer::Overlay(overlay) => overlay.pre_render(context, transform),
            RootLayer::Message(message) => message.pre_render(context, transform),
            RootLayer::Screen(screen) => screen.pre_render(context, transform),
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
            // RootLayer::Overlay(overlay) => overlay.render(pass, transform, stencil_ref, pass_kind),
            RootLayer::Message(message) => message.render(pass, transform, stencil_ref, pass_kind),
            RootLayer::Screen(screen) => screen.render(pass, transform, stencil_ref, pass_kind),
        }
    }
}

impl DrawableLayer for RootLayer {
    fn properties(&self) -> &LayerProperties {
        match self {
            // RootLayer::Overlay(overlay) => overlay.properties(),
            RootLayer::Message(message) => message.properties(),
            RootLayer::Screen(screen) => screen.properties(),
        }
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        match self {
            // RootLayer::Overlay(overlay) => overlay.properties_mut(),
            RootLayer::Message(message) => message.properties_mut(),
            RootLayer::Screen(screen) => screen.properties_mut(),
        }
    }
}

#[derive(RenderClone)]
pub struct RootLayerGroup {
    #[render_clone(needs_render)]
    inner: LayerGroup<RootLayer>,
}

impl RootLayerGroup {
    pub fn new(
        adv_fonts: AdvFonts,
        messagebox_textures: Arc<MessageboxTextures>,
        voice_player: VoicePlayer,
    ) -> Self {
        let mut inner = LayerGroup::new(Some("RootLayerGroup".to_string()));
        inner.add_layer(
            MESSAGE_LAYERBANK,
            MessageLayer::new(adv_fonts, messagebox_textures, voice_player).into(),
        );
        inner.add_layer(SCREEN_LAYERBANK, ScreenLayer::new(4).into());

        Self { inner }
    }

    pub fn screen_layer(&self) -> &ScreenLayer {
        let Some(RootLayer::Screen(screen)) = self.inner.get_layer(SCREEN_LAYERBANK) else {
            unreachable!();
        };

        screen
    }

    pub fn screen_layer_mut(&mut self) -> &mut ScreenLayer {
        let Some(RootLayer::Screen(screen)) = self.inner.get_layer_mut(SCREEN_LAYERBANK) else {
            unreachable!();
        };

        screen
    }

    pub fn message_layer(&self) -> &MessageLayer {
        let Some(RootLayer::Message(message)) = self.inner.get_layer(MESSAGE_LAYERBANK) else {
            unreachable!();
        };

        message
    }

    pub fn message_layer_mut(&mut self) -> &mut MessageLayer {
        let Some(RootLayer::Message(message)) = self.inner.get_layer_mut(MESSAGE_LAYERBANK) else {
            unreachable!();
        };

        message
    }
}

impl AdvUpdatable for RootLayerGroup {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.inner.update(context);
    }
}

impl Layer for RootLayerGroup {
    fn get_stencil_bump(&self) -> u8 {
        self.inner.get_stencil_bump()
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        self.inner.pre_render(context, transform);
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        self.inner.render(pass, transform, stencil_ref, pass_kind);
    }
}

impl DrawableLayer for RootLayerGroup {
    fn properties(&self) -> &LayerProperties {
        self.inner.properties()
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        self.inner.properties_mut()
    }
}
