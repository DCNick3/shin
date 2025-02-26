use replace_with::replace_with;
use shin_core::primitives::color::{FloatColor4, UnormColor};
use shin_render::{
    PassKind, RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{
        RenderClone, RenderCloneCtx,
        texture::{DepthStencilTarget, TextureTarget},
    },
};
use tracing::debug;

use crate::{
    layer::{
        DrawableLayer, Layer, NewDrawableLayer,
        either::EitherLayer,
        new_drawable_layer::{NewDrawableLayerNeedsSeparatePass, NewDrawableLayerState},
        page_layer::PageLayer,
        properties::LayerProperties,
        render_layer,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
    },
    render::{PreRenderContext, render_texture_holder::RenderTextureHolder},
    update::{AdvUpdatable, AdvUpdateContext},
    wiper::{AnyWiper, Wiper as _},
};

#[derive(RenderClone)]
struct TransitionLayer {
    #[render_clone(needs_render)]
    source_layer: Option<EitherLayer<Box<PageLayer>, Box<TransitionLayer>>>,
    #[render_clone(needs_render)]
    target_layer: Option<PageLayer>,
    #[render_clone(needs_render)]
    wiper: Option<AnyWiper>,

    #[render_clone(needs_render)]
    source_render_texture: RenderTextureHolder,
    #[render_clone(needs_render)]
    target_render_texture: RenderTextureHolder,
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
            source_render_texture: RenderTextureHolder::new(
                "TransitionLayer/source_render_texture",
            ),
            target_render_texture: RenderTextureHolder::new(
                "TransitionLayer/target_render_texture",
            ),
        }
    }

    pub fn dummy() -> Self {
        Self {
            source_layer: None,
            target_layer: None,
            wiper: None,
            source_render_texture: RenderTextureHolder::new(
                "TransitionLayer/source_render_texture",
            ),
            target_render_texture: RenderTextureHolder::new(
                "TransitionLayer/target_render_texture",
            ),
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
                self.source_render_texture.clear();
                self.target_render_texture.clear();
            }
        }
    }
}

impl Layer for TransitionLayer {
    fn fast_forward(&mut self) {
        if let Some(target_layer) = &mut self.target_layer {
            target_layer.fast_forward();
        }
        self.get_target_layer_mut().fast_forward();
        if let Some(wiper) = &mut self.wiper {
            wiper.fast_forward();
        }
    }

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

        let source_render_texture = self.source_render_texture.get_or_init(context);

        {
            let mut pass = context.begin_pass(
                source_render_texture.as_texture_target(),
                Some(context.depth_stencil),
                "TransitionLayer/source",
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

        let target_render_texture = self.target_render_texture.get_or_init(context);
        {
            let mut pass = context.begin_pass(
                target_render_texture.as_texture_target(),
                Some(context.depth_stencil),
                "TransitionLayer/target",
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
                    .get()
                    .unwrap()
                    .as_texture_source(),
                self.source_render_texture
                    .get()
                    .unwrap()
                    .as_texture_source(),
            );
        }
    }
}

#[derive(RenderClone)]
pub struct ScreenLayer {
    #[render_clone(needs_render)]
    active_layer: TransitionLayer,
    #[render_clone(needs_render)]
    pending_layer: Option<PageLayer>,

    plane_count: usize,

    #[render_clone(needs_render)]
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

    pub fn pageback(&mut self, ctx: &mut RenderCloneCtx, start_anew: bool) {
        if start_anew {
            todo!()
        }

        self.pending_layer = Some(self.active_layer.get_target_layer().render_clone(ctx));

        // NB: the original engine iterates over plane `LayerGroup`s and calls `LayerGroup::stop_transition` on them
        // We do not implement `LayerGroup`-level transitions, so this is skipped
    }

    pub fn apply_transition(&mut self, wiper: Option<AnyWiper>) {
        let Some(pending_layer) = self.pending_layer.take() else {
            return;
        };
        replace_with(
            &mut self.active_layer,
            TransitionLayer::dummy,
            |prev_transition_layer| {
                TransitionLayer::new(Some(Box::new(prev_transition_layer)), pending_layer, wiper)
            },
        );
    }

    pub fn is_transition_active(&self) -> bool {
        self.active_layer.is_transition_active()
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
        let mut pass = context.begin_pass(target, Some(depth_stencil), "ScreenLayer/indirect");

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
    fn fast_forward(&mut self) {
        self.props.fast_forward();
        self.active_layer.fast_forward();
        if let Some(target_layer) = &mut self.pending_layer {
            target_layer.fast_forward();
        }
    }

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
