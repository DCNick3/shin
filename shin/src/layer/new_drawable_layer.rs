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
    update::{Updatable, UpdateContext},
};

pub trait NewDrawableLayerNeedsSeparatePass {
    fn needs_separate_pass(&self, _properties: &LayerProperties) -> bool {
        false
    }
}

pub trait NewDrawableLayer: NewDrawableLayerNeedsSeparatePass {
    #[expect(unused)] // it will be used. eventually.
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        properties: &LayerProperties,
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
    #[expect(unused)]
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
        let Some(tex) = self.render_texture_src.as_ref() else {
            return None;
        };

        Some(PrerenderedDrawable {
            render_texture: tex.as_texture_source(),
            target_pass: self.target_pass,
        })
    }

    pub fn update(&mut self, _context: &UpdateContext) {
        // TODO
    }

    pub fn is_rendered_directly<T: NewDrawableLayerNeedsSeparatePass>(
        &self,
        properties: &LayerProperties,
        delegate: &T,
    ) -> bool {
        let Some(_tex) = self.get_prerendered_tex() else {
            return true;
        };

        if delegate.needs_separate_pass(properties) {
            return false;
        }

        todo!("check tex.force_transparent_pass")
    }

    pub fn pre_render<T: NewDrawableLayer>(
        &mut self,
        context: &mut PreRenderContext,
        properties: &LayerProperties,
        delegate: &mut T,
        transform: &TransformParams,
    ) {
        if !properties.is_visible() {
            return;
        }

        let blur_radius = properties.get_value(LayerProperty::BlurRadius) * 0.001;
        let prop70 = properties.get_value(LayerProperty::Prop70) * 0.001;
        let mosaic_size = properties.get_value(LayerProperty::MosaicSize) as i32;
        let raster_horizontal_amplitude =
            properties.get_value(LayerProperty::RasterHorizontalAmplitude);
        let raster_vertical_amplitude =
            properties.get_value(LayerProperty::RasterVerticalAmplitude);
        let ripple_amplitude = properties.get_value(LayerProperty::RippleAmplitude);
        let dissolve_intensity = properties.get_value(LayerProperty::DissolveIntensity) * 0.001;
        let ghosting_alpha = properties.get_value(LayerProperty::GhostingAlpha) * 0.001;

        if blur_radius.abs() < f32::EPSILON
            && prop70 < f32::EPSILON
            && mosaic_size <= 0
            && raster_horizontal_amplitude.abs() < f32::EPSILON
            && raster_vertical_amplitude.abs() < f32::EPSILON
            && ripple_amplitude.abs() < f32::EPSILON
            && dissolve_intensity <= 0.0
            && ghosting_alpha <= 0.0
            && !delegate.needs_separate_pass(properties)
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
            properties,
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
        properties: &LayerProperties,
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

        let color_multiplier = properties.get_color_multiplier().premultiply();
        let blend_type = properties.get_blend_type();
        let fragment_shader = properties.get_fragment_shader();
        let fragment_shader_param = properties.get_fragment_shader_param();

        // NOTE: the transform is actually used just for clipping
        // we still compute it just in case
        let mut self_transform = properties.get_transform_params();
        self_transform.compose_with(transform, properties.get_compose_flags());

        let clip_params = properties.get_clip_params();
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
        properties: &LayerProperties,
        delegate: &T,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        // TODO: implement the indirect drawing stuff
        if !properties.is_visible() {
            return;
        }

        let mut self_transform = properties.get_transform_params();
        self_transform.compose_with(transform, properties.get_compose_flags());

        let drawable = properties.get_drawable_params();
        let clip = properties.get_clip_params();

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

impl<T: Updatable> Updatable for NewDrawableLayerWrapper<T> {
    fn update(&mut self, context: &UpdateContext) {
        self.inner_layer.update(context);
        self.state.update(context);
        self.props.update(context);
    }
}

impl<T: NewDrawableLayer + Clone + Updatable> Layer for NewDrawableLayerWrapper<T> {
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

impl<T: NewDrawableLayer + Clone + Updatable> DrawableLayer for NewDrawableLayerWrapper<T> {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
