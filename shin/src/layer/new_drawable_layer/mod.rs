mod effect_passes;

use shin_core::vm::command::types::LayerProperty;
use shin_render::{
    ColorBlendType, DepthStencilState, DrawPrimitive, LayerShaderOutputKind, PassKind,
    RenderProgramWithArguments, RenderRequestBuilder, StencilFunction, StencilOperation,
    StencilPipelineState, StencilState,
    quad_vertices::build_quad_vertices,
    render_pass::RenderPass,
    render_texture::RenderTexture,
    shaders::types::{
        RenderClone,
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureSource, TextureTarget},
        vertices::PosTexVertex,
    },
};

use crate::{
    layer::{
        DrawableLayer, Layer, LayerProperties,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
    },
    render::{
        PreRenderContext, VIRTUAL_CANVAS_SIZE_VEC, render_texture_holder::RenderTextureHolder,
        top_left_projection_matrix,
    },
    update::{AdvUpdatable, AdvUpdateContext},
};

pub trait NewDrawableLayerNeedsSeparatePass {
    fn needs_separate_pass(&self, #[expect(unused)] props: &LayerProperties) -> bool {
        false
    }
}

pub trait NewDrawableLayer: NewDrawableLayerNeedsSeparatePass {
    #[tracing::instrument(skip_all)]
    #[expect(unused)] // it will be used. eventually.
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        props: &LayerProperties,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
        transform: &TransformParams,
    ) -> PassKind {
        // TODO: initiate a generic render pass and delegate to Self::render_drawable_direct
        todo!()
    }
    fn render_drawable_direct(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        drawable: &DrawableParams,
        clip: &DrawableClipParams,
        // TODO: make a strong type for this?
        stencil_ref: u8,
        pass_kind: PassKind,
    );
}

pub trait NewDrawableLayerFastForward {
    fn fast_forward(&mut self);
}

pub struct PrerenderedDrawable<'a> {
    pub render_texture: TextureSource<'a>,
    pub target_pass: PassKind,
}

#[derive(Debug, RenderClone)]
pub struct NewDrawableLayerState {
    #[render_clone(needs_render)]
    render_texture_src: RenderTextureHolder,
    // it's needed for any kind of effect. we just don't have them implemented yet
    #[render_clone(needs_render)]
    render_texture_target: Option<RenderTexture>,
    #[render_clone(needs_render)]
    render_texture_prev_frame: Option<RenderTexture>,
    target_pass: PassKind,
}

impl NewDrawableLayerState {
    pub fn new() -> Self {
        Self {
            render_texture_src: RenderTextureHolder::new("NewDrawableLayerState/render_texture"),
            render_texture_target: None,
            render_texture_prev_frame: None,
            target_pass: PassKind::Transparent,
        }
    }

    pub fn get_prerendered_tex(&self) -> Option<PrerenderedDrawable> {
        let tex = self.render_texture_src.get()?;

        Some(PrerenderedDrawable {
            render_texture: tex.as_texture_source(),
            target_pass: self.target_pass,
        })
    }

    pub fn update(&mut self, _context: &AdvUpdateContext) {
        // TODO: there are several float values we need to track and to update for some effects
    }

    pub fn is_rendered_opaquely<T: NewDrawableLayerNeedsSeparatePass>(
        &self,
        props: &LayerProperties,
        delegate: &T,
    ) -> bool {
        let Some(tex) = self.get_prerendered_tex() else {
            return true;
        };

        if delegate.needs_separate_pass(props) {
            // weird! I think this is too conservative
            return false;
        }

        tex.target_pass == PassKind::Opaque
    }

    pub fn pre_render<T: NewDrawableLayer>(
        &mut self,
        context: &mut PreRenderContext,
        props: &LayerProperties,
        delegate: &mut T,
        transform: &TransformParams,
    ) {
        if !props.is_visible() {
            return;
        }

        let blur_radius = props.get_value(LayerProperty::BlurRadius) * 0.001;
        let prop70 = props.get_value(LayerProperty::Prop70) * 0.001;
        let mosaic_size = props.get_value(LayerProperty::MosaicSize) as i32;
        let raster_horizontal_amplitude = props.get_value(LayerProperty::RasterHorizontalAmplitude);
        let raster_vertical_amplitude = props.get_value(LayerProperty::RasterVerticalAmplitude);
        let ripple_amplitude = props.get_value(LayerProperty::RippleAmplitude);
        let dissolve_intensity = props.get_value(LayerProperty::DissolveIntensity) * 0.001;
        let ghosting_alpha = props.get_value(LayerProperty::GhostingAlpha) * 0.001;

        if blur_radius.abs() < f32::EPSILON
            && prop70 < f32::EPSILON
            && mosaic_size <= 0
            && raster_horizontal_amplitude.abs() < f32::EPSILON
            && raster_vertical_amplitude.abs() < f32::EPSILON
            && ripple_amplitude.abs() < f32::EPSILON
            && dissolve_intensity <= 0.0
            && ghosting_alpha <= 0.0
            && !delegate.needs_separate_pass(props)
        {
            self.render_texture_src.clear();
            self.render_texture_target = None;
            self.render_texture_prev_frame = None;
            return;
        }

        if ghosting_alpha <= 0.0 {
            self.render_texture_prev_frame = None;
        } else {
            std::mem::swap(
                &mut self.render_texture_prev_frame,
                &mut self.render_texture_src.as_inner_mut(),
            );
        }

        let render_texture_src = self.render_texture_src.get_or_init(context);
        self.target_pass = delegate.render_drawable_indirect(
            context,
            props,
            render_texture_src.as_texture_target(),
            context.depth_stencil,
            transform,
        );

        if blur_radius.abs() >= f32::EPSILON {
            todo!()
        }
        if prop70 >= f32::EPSILON {
            todo!()
        }
        if mosaic_size > 0 {
            todo!()
        }
        if raster_horizontal_amplitude.abs() >= f32::EPSILON
            || raster_vertical_amplitude.abs() >= f32::EPSILON
        {
            todo!()
        }
        if ripple_amplitude.abs() >= f32::EPSILON {
            todo!()
        }
        if dissolve_intensity > 0.0 {
            todo!()
        }
        match (ghosting_alpha > 0.0, &mut self.render_texture_prev_frame) {
            (true, Some(render_texture_prev_frame)) => {
                effect_passes::apply_ghosting(
                    context,
                    props,
                    render_texture_src,
                    render_texture_prev_frame,
                    ghosting_alpha,
                );
            }
            _ => self.render_texture_prev_frame = None,
        }
    }

    pub fn try_finish_indirect_render(
        &self,
        props: &LayerProperties,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) -> bool {
        let Some(tex) = self.render_texture_src.get() else {
            return false;
        };

        if pass_kind != self.target_pass {
            return true;
        }

        let color_multiplier = props.get_color_multiplier().premultiply();
        let blend_type = props.get_blend_type();
        let fragment_shader = props.get_fragment_shader();
        let fragment_shader_param = props.get_fragment_shader_param();

        // NOTE: the transform is actually used just for clipping
        // we still compute it just in case
        let _self_transform = props.get_composed_transform_params(transform);

        let clip_params = props.get_clip_params();
        assert_eq!(clip_params.mode, DrawableClipMode::None);

        let transform = top_left_projection_matrix();

        let vertices = &build_quad_vertices(|t| PosTexVertex {
            position: t * VIRTUAL_CANVAS_SIZE_VEC,
            texture_position: t,
        });

        pass.run(
            RenderRequestBuilder::new()
                .depth_stencil(DepthStencilState {
                    depth: Default::default(),
                    stencil: StencilState {
                        pipeline: StencilPipelineState {
                            function: StencilFunction::Greater,
                            stencil_fail_operation: StencilOperation::Keep,
                            depth_fail_operation: StencilOperation::Keep,
                            pass_operation: StencilOperation::Replace,
                            ..Default::default()
                        },
                        stencil_reference: stencil_ref,
                    },
                })
                .color_blend_type(match pass_kind {
                    PassKind::Opaque => ColorBlendType::Opaque,
                    PassKind::Transparent => ColorBlendType::from_premultiplied_layer(blend_type),
                })
                .build(
                    RenderProgramWithArguments::Layer {
                        output_kind: match pass_kind {
                            PassKind::Opaque => LayerShaderOutputKind::Layer,
                            PassKind::Transparent => LayerShaderOutputKind::LayerDiscard,
                        },
                        fragment_shader,
                        vertices: VertexSource::VertexData { vertices },
                        texture: tex.as_texture_source(),
                        transform,
                        color_multiplier,
                        fragment_shader_param,
                    },
                    DrawPrimitive::TrianglesStrip,
                ),
        );

        true
    }

    #[tracing::instrument(skip_all)]
    pub fn render<T: NewDrawableLayer>(
        &self,
        props: &LayerProperties,
        delegate: &T,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        // TODO: implement the indirect drawing stuff
        if !props.is_visible() {
            return;
        }

        let self_transform = props.get_composed_transform_params(transform);

        let drawable = props.get_drawable_params();
        let clip = props.get_clip_params();

        delegate.render_drawable_direct(
            pass,
            &self_transform,
            &drawable,
            &clip,
            stencil_ref,
            pass_kind,
        );
    }
}

// packages NewDrawableLayerState and LayerProperties to implement simple NewDrawable-based layers
#[derive(Debug, RenderClone)]
pub struct NewDrawableLayerWrapper<T> {
    #[render_clone(needs_render)]
    inner_layer: T,
    #[render_clone(needs_render)]
    state: NewDrawableLayerState,
    props: LayerProperties,
}

impl<T: NewDrawableLayer> NewDrawableLayerWrapper<T> {
    pub fn from_inner(inner_layer: T) -> Self {
        Self {
            inner_layer,
            state: NewDrawableLayerState::new(),
            props: LayerProperties::new(),
        }
    }

    pub fn inner_ref(&self) -> &T {
        &self.inner_layer
    }
}

impl<T: AdvUpdatable> AdvUpdatable for NewDrawableLayerWrapper<T> {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.inner_layer.update(context);
        self.state.update(context);
        self.props.update(context);
    }
}

impl<T: NewDrawableLayer + AdvUpdatable + NewDrawableLayerFastForward> Layer
    for NewDrawableLayerWrapper<T>
{
    fn fast_forward(&mut self) {
        self.props.fast_forward();
        self.inner_layer.fast_forward();
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        self.state
            .pre_render(context, &self.props, &mut self.inner_layer, transform);
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        self.state.render(
            &self.props,
            &self.inner_layer,
            pass,
            transform,
            stencil_ref,
            pass_kind,
        );
    }
}

impl<T: NewDrawableLayer + AdvUpdatable + NewDrawableLayerFastForward> DrawableLayer
    for NewDrawableLayerWrapper<T>
{
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
