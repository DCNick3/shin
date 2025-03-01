mod either;
mod layer_group;
pub mod message_layer;
mod new_drawable_layer;
mod page_layer;
mod properties;
pub mod render_params;
#[expect(unused)]
mod root_layer_group;
mod screen_layer;
pub mod user;
mod wobbler;

use derive_more::From;
use glam::vec3;
pub use layer_group::LayerGroup;
pub use new_drawable_layer::{NewDrawableLayer, NewDrawableLayerWrapper};
pub use page_layer::PageLayer;
pub use properties::{LayerProperties, LayerPropertiesState};
pub use root_layer_group::RootLayerGroup;
pub use screen_layer::ScreenLayer;
use shin_core::primitives::color::FloatColor4;
use shin_render::{
    DepthStencilState, DrawPrimitive, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    StencilFunction, StencilOperation, StencilPipelineState, StencilState,
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, vertices::PosVertex},
};

use crate::{
    layer::{render_params::TransformParams, user::UserLayer},
    render::PreRenderContext,
    update::AdvUpdatable,
};

pub trait Layer: AdvUpdatable {
    fn fast_forward(&mut self);
    fn get_stencil_bump(&self) -> u8 {
        1
    }
    fn pre_render(
        &mut self,
        #[expect(unused)] context: &mut PreRenderContext,
        #[expect(unused)] transform: &TransformParams,
    ) {
    }
    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    );
}

impl<T: Layer> Layer for Box<T> {
    #[inline]
    fn fast_forward(&mut self) {
        (**self).fast_forward()
    }

    #[inline]
    fn get_stencil_bump(&self) -> u8 {
        (**self).get_stencil_bump()
    }

    #[inline]
    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        (**self).pre_render(context, transform)
    }

    #[inline]
    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        (**self).render(pass, transform, stencil_ref, pass_kind)
    }
}

/// A layer that has properties
///
/// Yes, really, this trait doesn't have anything to do with drawing.
pub trait DrawableLayer: Layer {
    // fn init(&mut self);
    // fn set_properties(&mut self, properties: LayerProperties);
    fn properties(&self) -> &LayerProperties;
    fn properties_mut(&mut self) -> &mut LayerProperties;
}

impl<T: DrawableLayer> DrawableLayer for Box<T> {
    #[inline]
    fn properties(&self) -> &LayerProperties {
        (**self).properties()
    }

    #[inline]
    fn properties_mut(&mut self) -> &mut LayerProperties {
        (**self).properties_mut()
    }
}

#[derive(From)]
pub enum AnyLayer<'a> {
    UserLayer(&'a UserLayer),
    RootLayerGroup(&'a RootLayerGroup),
    ScreenLayer(&'a ScreenLayer),
    PageLayer(&'a PageLayer),
    LayerGroup(&'a LayerGroup),
}

impl AnyLayer<'_> {
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

impl AnyLayerMut<'_> {
    #[expect(unused)]
    pub fn properties(&self) -> &LayerProperties {
        match self {
            Self::UserLayer(layer) => layer.properties(),
            Self::RootLayerGroup(layer) => layer.properties(),
            Self::ScreenLayer(layer) => layer.properties(),
            Self::PageLayer(layer) => layer.properties(),
            Self::LayerGroup(layer) => layer.properties(),
        }
    }

    #[expect(unused)]
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

#[tracing::instrument(skip_all)]
pub fn render_layers_default_cb(
    color: FloatColor4,
) -> impl Fn(&mut RenderPass, &TransformParams, u8) {
    move |pass, _transform, stencil_ref| {
        let vertices = &[
            PosVertex {
                position: vec3(-1.0, 1.0, 0.0),
            },
            PosVertex {
                position: vec3(3.0, 1.0, 0.0),
            },
            PosVertex {
                position: vec3(-1.0, -3.0, 0.0),
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
                .build(
                    RenderProgramWithArguments::Clear {
                        vertices: VertexSource::VertexData { vertices },
                        color,
                    },
                    DrawPrimitive::Triangles,
                ),
        );
    }
}

#[tracing::instrument(skip_all)]
#[inline]
pub fn render_layers_with_bg<F>(
    pass: &mut RenderPass,
    transform: &TransformParams,
    // TODO: maybe use AnyLayer here?
    layers: &[&dyn Layer],
    render_bg_cb: F,
    is_render_bg_cb_noop: bool,
    mut stencil_ref: u8,
) -> u8
where
    F: Fn(&mut RenderPass, &TransformParams, u8),
{
    let mut render_items = Vec::with_capacity(layers.len());

    if stencil_ref == 0 {
        stencil_ref = 1;
    }

    let orig_stencil_ref = stencil_ref;

    if !is_render_bg_cb_noop {
        stencil_ref += 1;
    }

    for &layer in layers {
        render_items.push((layer, stencil_ref));
        stencil_ref += layer.get_stencil_bump();
    }

    for &(layer, stencil_ref) in render_items.iter().rev() {
        layer.render(pass, transform, stencil_ref, PassKind::Opaque);
    }

    render_bg_cb(pass, transform, orig_stencil_ref);

    for &(layer, stencil_ref) in &render_items {
        layer.render(pass, transform, stencil_ref, PassKind::Transparent);
    }

    stencil_ref
}

#[tracing::instrument(skip_all)]
pub fn render_layer_without_bg(
    pass: &mut RenderPass,
    transform: &TransformParams,
    layers: &dyn Layer,
    stencil_ref: u8,
) {
    render_layers_with_bg(pass, transform, &[layers], |_, _, _| {}, true, stencil_ref);
}

#[expect(unused)] // for future stuff
pub fn render_layers(
    pass: &mut RenderPass,
    transform: &TransformParams,
    layers: &[&dyn Layer],
    color: FloatColor4,
    stencil_ref: u8,
) {
    render_layers_with_bg(
        pass,
        transform,
        layers,
        render_layers_default_cb(color),
        false,
        stencil_ref,
    );
}

pub fn render_layer(
    pass: &mut RenderPass,
    transform: &TransformParams,
    layer: &dyn Layer,
    background_color: FloatColor4,
    stencil_ref: u8,
) {
    render_layers_with_bg(
        pass,
        transform,
        &[layer],
        render_layers_default_cb(background_color),
        false,
        stencil_ref,
    );
}
