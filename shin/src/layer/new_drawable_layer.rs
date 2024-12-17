use glam::vec4;
use shin_core::vm::command::types::LayerProperty;
use shin_render::{
    render_pass::RenderPass,
    render_texture::RenderTexture,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureSource, TextureTarget},
        vertices::LayerVertex,
    },
    shin_orthographic_projection_matrix, ColorBlendType, DepthStencilState, DrawPrimitive,
    LayerShaderOutputKind, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    StencilFunction, StencilOperation, StencilPipelineState, StencilState,
};

use crate::{
    layer::{
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        DrawableLayer, Layer, LayerProperties, PreRenderContext,
    },
    update::{AdvUpdatable, AdvUpdateContext},
};

pub trait NewDrawableLayerNeedsSeparatePass {
    fn needs_separate_pass(&self, #[expect(unused)] props: &LayerProperties) -> bool {
        false
    }
}

pub trait NewDrawableLayer: NewDrawableLayerNeedsSeparatePass {
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

#[expect(unused)] // this will be used once mask rendering in LayerGroup will be implemented
pub struct PrerenderedDrawable<'a> {
    render_texture: TextureSource<'a>,
    target_pass: PassKind,
}

#[derive(Debug, Clone)]
pub struct NewDrawableLayerState {
    render_texture_src: Option<RenderTexture>,
    // it's needed for any kind of effect. we just don't have them implemented yet
    render_texture_target: Option<RenderTexture>,
    render_texture_prev_frame: Option<RenderTexture>,
    target_pass: PassKind,
}

impl NewDrawableLayerState {
    pub fn new() -> Self {
        Self {
            render_texture_src: None,
            render_texture_target: None,
            render_texture_prev_frame: None,
            target_pass: PassKind::Transparent,
        }
    }

    pub fn get_prerendered_tex(&self) -> Option<PrerenderedDrawable> {
        let tex = self.render_texture_src.as_ref()?;

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
            self.render_texture_target = None;
            self.render_texture_src = None;
            self.render_texture_prev_frame = None;
            return;
        }

        if ghosting_alpha <= 0.0 {
            self.render_texture_prev_frame = None;
        } else {
            // TODO: preserve render_texture_src as render_texture_prev_frame, while re-using render_texture_prev_frame as render_texture_src
            todo!()
        }

        let render_texture_src = context.ensure_render_texture(&mut self.render_texture_src);
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
        if ghosting_alpha <= 0.0 || self.render_texture_prev_frame.is_none() {
            self.render_texture_prev_frame = None;
        } else {
            todo!()
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
        let Some(tex) = &self.render_texture_src else {
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

        let transform = shin_orthographic_projection_matrix(0.0, 1920.0, 1080.0, 0.0, -1.0, 1.0);

        let vertices = &[
            LayerVertex {
                coords: vec4(0.0, 0.0, 0.0, 0.0),
            },
            LayerVertex {
                coords: vec4(1920.0, 0.0, 1.0, 0.0),
            },
            LayerVertex {
                coords: vec4(0.0, 1080.0, 0.0, 1.0),
            },
            LayerVertex {
                coords: vec4(1920.0, 1080.0, 1.0, 1.0),
            },
        ];

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
#[derive(Debug, Clone)]
pub struct NewDrawableLayerWrapper<T> {
    inner_layer: T,
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
}

impl<T: AdvUpdatable> AdvUpdatable for NewDrawableLayerWrapper<T> {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.inner_layer.update(context);
        self.state.update(context);
        self.props.update(context);
    }
}

impl<T: NewDrawableLayer + Clone + AdvUpdatable> Layer for NewDrawableLayerWrapper<T> {
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

impl<T: NewDrawableLayer + Clone + AdvUpdatable> DrawableLayer for NewDrawableLayerWrapper<T> {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
