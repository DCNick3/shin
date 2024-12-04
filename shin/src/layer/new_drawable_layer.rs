use shin_core::vm::command::types::LayerProperty;
use shin_render::{render_pass::RenderPass, PassKind};

use crate::layer::{
    render_params::{DrawableClipParams, DrawableParams, TransformParams},
    LayerProperties,
};

pub trait DrawableLayer {
    // fn init(&mut self);
    // fn set_properties(&mut self, properties: LayerProperties);
    fn get_properties(&self) -> &LayerProperties;
}

pub trait NewDrawableLayer: DrawableLayer {
    fn needs_separate_pass(&self) -> bool {
        false
    }
    fn render_drawable_indirect(&self) {
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

pub struct NewDrawableLayerWrapper<T> {
    inner_layer: T,
}

impl<T: NewDrawableLayer> NewDrawableLayerWrapper<T> {
    pub fn new(inner_layer: T) -> Self {
        Self { inner_layer }
    }

    pub fn pre_render(&mut self) {
        let properties = self.inner_layer.get_properties();
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
            && !self.inner_layer.needs_separate_pass()
        {
            return;
        }

        todo!("implement NewDrawableLayerWrapper effects");
    }

    pub fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        // TODO: implement the indirect drawing stuff

        let properties = self.inner_layer.get_properties();
        if !properties.is_visible() {
            return;
        }

        let mut self_transform = properties.get_transform_params();
        self_transform.compose_with(transform, properties.get_compose_flags());

        let drawable = properties.get_drawable_params();
        let clip = properties.get_clip_params();

        self.inner_layer.render_drawable_direct(
            pass,
            &self_transform,
            &drawable,
            &clip,
            stencil_ref,
            pass_kind,
        );
    }
}
