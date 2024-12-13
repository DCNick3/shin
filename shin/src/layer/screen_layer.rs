use shin_render::{
    render_pass::RenderPass,
    render_texture::RenderTexture,
    shaders::types::{
        texture::{DepthStencilTarget, TextureTarget},
        vertices::{FloatColor4, UnormColor},
    },
    PassKind, RenderRequestBuilder,
};
use tracing::debug;

use crate::{
    layer::{
        either::EitherLayer,
        new_drawable_layer::{NewDrawableLayerNeedsSeparatePass, NewDrawableLayerState},
        page_layer::PageLayer,
        properties::LayerProperties,
        render_layer,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        DrawableLayer, Layer, NewDrawableLayer, PreRenderContext,
    },
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::{AnyWiper, Wiper as _},
};

#[derive(Clone)]
struct TransitionLayer {
    source_layer: Option<EitherLayer<Box<PageLayer>, Box<TransitionLayer>>>,
    target_layer: Option<PageLayer>,
    wiper: Option<AnyWiper>,

    source_render_texture: Option<RenderTexture>,
    target_render_texture: Option<RenderTexture>,
}

impl TransitionLayer {
    pub fn new(
        source: Option<Box<TransitionLayer>>,
        target: PageLayer,
        wiper: Option<AnyWiper>,
    ) -> Self {
        Self {
            source_layer: source.map(EitherLayer::Right),
            target_layer: Some(target),
            wiper,
            source_render_texture: None,
            target_render_texture: None,
        }
    }

    pub fn dummy() -> Self {
        Self {
            source_layer: None,
            target_layer: None,
            wiper: None,
            source_render_texture: None,
            target_render_texture: None,
        }
    }

    pub fn is_transition_active(&self) -> bool {
        self.wiper.is_some()
    }

    pub fn get_target_layer(&self) -> &PageLayer {
        self.target_layer.as_ref().unwrap()
    }

    pub fn get_target_layer_mut(&mut self) -> &mut PageLayer {
        self.target_layer.as_mut().unwrap()
    }

    pub fn into_target_layer(self) -> PageLayer {
        self.target_layer.unwrap()
    }
}

impl AdvUpdatable for TransitionLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
        if let Some(source_layer) = &mut self.source_layer {
            source_layer.update(context);

            // if we are storing a finished transition layer, flatten the hierarchy
            if let EitherLayer::Right(transition_layer) = source_layer {
                transition_layer.update(context);
                if !transition_layer.is_transition_active() {
                    debug!("Inner TransitionLayer has finished running, flattening hierarchy");
                    let Some(EitherLayer::Right(transition_layer)) = self.source_layer.take()
                    else {
                        unreachable!()
                    };
                    self.source_layer = Some(EitherLayer::Left(Box::new(
                        transition_layer.into_target_layer(),
                    )));
                }
            }
        }

        self.get_target_layer_mut().update(context);

        if let Some(wiper) = &mut self.wiper {
            wiper.update(context);
            if !wiper.is_running() {
                debug!("Wiper has finished running, cleaning up & switching to direct rendering");
                self.wiper = None;
                self.source_layer = None;
                self.source_render_texture = None;
                self.target_render_texture = None;
            }
        }
    }
}

impl Layer for TransitionLayer {
    fn get_stencil_bump(&self) -> u8 {
        if self.wiper.is_some() {
            return 1;
        }

        self.get_target_layer().get_stencil_bump()
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        if self.wiper.is_none() {
            self.get_target_layer_mut().pre_render(context, transform);
            return;
        }

        self.source_layer
            .as_mut()
            .unwrap()
            .pre_render(context, transform);
        self.target_layer
            .as_mut()
            .unwrap()
            .pre_render(context, transform);

        let source_render_texture = context.ensure_render_texture(&mut self.source_render_texture);

        {
            let mut pass = context.begin_pass(
                source_render_texture.as_texture_target(),
                context.depth_stencil,
            );
            pass.clear(None, Some(0), None);

            render_layer(
                &mut pass,
                transform,
                self.source_layer.as_mut().unwrap(),
                FloatColor4::BLACK,
                0,
            );
        }

        let target_render_texture = context.ensure_render_texture(&mut self.target_render_texture);
        {
            let mut pass = context.begin_pass(
                target_render_texture.as_texture_target(),
                context.depth_stencil,
            );
            pass.clear(None, Some(0), None);

            render_layer(
                &mut pass,
                transform,
                self.target_layer.as_mut().unwrap(),
                FloatColor4::BLACK,
                0,
            );
        }
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        let Some(wiper) = &self.wiper else {
            self.get_target_layer()
                .render(pass, transform, stencil_ref, pass_kind);
            return;
        };

        if pass_kind == PassKind::Opaque {
            let render_request_builder =
                RenderRequestBuilder::new().depth_stencil_shorthand(stencil_ref, true, false);

            wiper.render(
                pass,
                render_request_builder,
                self.target_render_texture
                    .as_ref()
                    .unwrap()
                    .as_texture_source(),
                self.source_render_texture
                    .as_ref()
                    .unwrap()
                    .as_texture_source(),
            );
        }
    }
}

#[derive(Clone)]
pub struct ScreenLayer {
    active_layer: TransitionLayer,
    pending_layer: Option<PageLayer>,

    #[expect(unused)] // for future stuff
    plane_count: usize,

    new_drawable_state: NewDrawableLayerState,
    props: LayerProperties,
}

impl ScreenLayer {
    pub fn new(plane_count: usize) -> Self {
        Self {
            // NB: the original game stores __either__ a `TransitionLayer` or `PageLayer` here
            // however, after the first transition, the `TransitionLayer` stays
            // I just decided to take a shorter route and always store `TransitionLayer` :)
            active_layer: TransitionLayer::new(None, PageLayer::new(plane_count), None),
            pending_layer: None,
            plane_count,
            new_drawable_state: NewDrawableLayerState::new(),
            props: LayerProperties::new(),
        }
    }

    pub fn page_layer(&self) -> &PageLayer {
        if let Some(pending_layer) = &self.pending_layer {
            pending_layer
        } else {
            self.active_layer.get_target_layer()
        }
    }

    pub fn page_layer_mut(&mut self) -> &mut PageLayer {
        if let Some(pending_layer) = &mut self.pending_layer {
            pending_layer
        } else {
            self.active_layer.get_target_layer_mut()
        }
    }

    #[expect(unused)] // for future stuff
    pub fn pageback(&mut self, immediate: bool) {
        if immediate {
            todo!()
        }

        let new_pending_layer = self.active_layer.clone().into_target_layer();

        self.pending_layer = Some(new_pending_layer);

        // NB: the original engine iterates over plane `LayerGroup`s and calls `LayerGroup::stop_transition` on them
        // We do not implement `LayerGroup`-level transitions, so this is skipped
    }

    #[expect(unused)] // for future stuff
    pub fn apply_transition(&mut self, wiper: Option<AnyWiper>) {
        let Some(pending_layer) = self.pending_layer.take() else {
            return;
        };

        // I am not a fan of this =(
        // unfortunately you can't just temporarily take the `active_layer` out in safe rust
        // so we have to invent a dummy value for it
        let prev_transition_layer =
            std::mem::replace(&mut self.active_layer, TransitionLayer::dummy());
        self.active_layer =
            TransitionLayer::new(Some(Box::new(prev_transition_layer)), pending_layer, wiper);
    }
}

struct ScreenLayerNewDrawableDelegate<'a> {
    active_layer: &'a TransitionLayer,
}

impl NewDrawableLayerNeedsSeparatePass for ScreenLayerNewDrawableDelegate<'_> {
    fn needs_separate_pass(&self, props: &LayerProperties) -> bool {
        props.get_clip_mode() != DrawableClipMode::None
            || props.is_fragment_shader_nontrivial()
            || props.is_blending_nontrivial()
    }
}

impl NewDrawableLayer for ScreenLayerNewDrawableDelegate<'_> {
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        props: &LayerProperties,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
        transform: &TransformParams,
    ) -> PassKind {
        let mut pass = context.begin_pass(target, depth_stencil);

        if !props.is_visible() {
            pass.clear(Some(UnormColor::BLACK), None, None);
        } else {
            let self_transform = props.get_composed_transform_params(transform);

            pass.clear(None, Some(0), None);
            // NB:     ^ doesn't clear color here
            // this is unlike PageLayer and LayerGroup, which do

            render_layer(
                &mut pass,
                &self_transform,
                self.active_layer,
                FloatColor4::BLACK,
                0,
            );
        }

        PassKind::Transparent
    }

    fn render_drawable_direct(
        &self,
        _pass: &mut RenderPass,
        _transform: &TransformParams,
        _drawable: &DrawableParams,
        _clip: &DrawableClipParams,
        _stencil_ref: u8,
        _pass_kind: PassKind,
    ) {
        // direct rendering is always done by the ScreenLayer without relying on NewDrawableLayer
        unreachable!()
    }
}

impl AdvUpdatable for ScreenLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.props.update(context);
        self.new_drawable_state.update(context);

        self.active_layer.update(context);
        if let Some(pending_layer) = &mut self.pending_layer {
            pending_layer.update(context);
        }
    }
}

impl Layer for ScreenLayer {
    fn get_stencil_bump(&self) -> u8 {
        self.active_layer.get_stencil_bump()
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        let props = &self.props;

        let self_transform = props.get_composed_transform_params(transform);

        self.active_layer.pre_render(context, &self_transform);

        let mut delegate = ScreenLayerNewDrawableDelegate {
            active_layer: &self.active_layer,
        };

        self.new_drawable_state
            .pre_render(context, props, &mut delegate, &self_transform);
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        let props = &self.props;
        if self.new_drawable_state.try_finish_indirect_render(
            props,
            pass,
            transform,
            stencil_ref,
            pass_kind,
        ) {
            return;
        }

        if !props.is_visible() {
            return;
        }

        let self_transform = props.get_composed_transform_params(transform);
        self.active_layer
            .render(pass, &self_transform, stencil_ref, pass_kind);
    }
}

impl DrawableLayer for ScreenLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
