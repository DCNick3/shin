mod layer_group;
#[expect(unused)]
mod message_layer;
mod new_drawable_layer;
mod page_layer;
mod properties;
pub mod render_params;
#[expect(unused)]
mod root_layer_group;
#[expect(unused)]
mod screen_layer;
pub mod user;
mod wobbler;

use std::sync::Arc;

use derive_more::From;
use glam::vec3;
pub use layer_group::LayerGroup;
pub use message_layer::MessageLayer;
pub use new_drawable_layer::{NewDrawableLayer, NewDrawableLayerWrapper};
pub use page_layer::PageLayer;
pub use properties::{LayerProperties, LayerPropertiesSnapshot};
pub use root_layer_group::RootLayerGroup;
pub use screen_layer::ScreenLayer;
use shin_render::{
    dynamic_buffer::DynamicBuffer,
    pipelines::PipelineStorage,
    render_pass::RenderPass,
    render_texture::RenderTexture,
    resize::SurfaceResizeSource,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureSamplerStore, TextureTarget},
        vertices::{FloatColor4, PosVertex},
    },
    DepthStencilState, DrawPrimitive, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    StencilFunction, StencilOperation, StencilPipelineState, StencilState,
};

use crate::{
    layer::{render_params::TransformParams, user::UserLayer},
    update::Updatable,
};

pub struct PreRenderContext<'immutable, 'pipelines, 'dynbuffer, 'encoder> {
    pub device: &'immutable Arc<wgpu::Device>,
    pub queue: &'immutable Arc<wgpu::Queue>,
    pub resize_source: &'immutable SurfaceResizeSource,
    pub sampler_store: &'immutable TextureSamplerStore,
    pub depth_stencil: DepthStencilTarget<'immutable>,

    pub pipeline_storage: &'pipelines mut PipelineStorage,
    pub dynamic_buffer: &'dynbuffer mut DynamicBuffer,
    pub encoder: &'encoder mut wgpu::CommandEncoder,
}

impl PreRenderContext<'_, '_, '_, '_> {
    pub fn new_render_texture(&self, label: Option<String>) -> RenderTexture {
        RenderTexture::new(
            self.device.clone(),
            self.queue.clone(),
            self.resize_source.handle(),
            label,
        )
    }

    pub fn ensure_render_texture<'a>(
        &self,
        storage: &'a mut Option<RenderTexture>,
    ) -> &'a mut RenderTexture {
        storage.get_or_insert_with(|| self.new_render_texture(None))
    }

    pub fn begin_pass(
        &mut self,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
    ) -> RenderPass {
        RenderPass::new(
            self.pipeline_storage,
            self.dynamic_buffer,
            self.sampler_store,
            self.device,
            self.encoder,
            target,
            depth_stencil,
            None,
        )
    }
}

pub trait Layer: Updatable {
    // fn fast_forward(&mut self);
    fn get_stencil_bump(&self) -> u8 {
        1
    }
    fn pre_render(&mut self, _context: &mut PreRenderContext, _transform: &TransformParams) {}
    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    );
}

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

pub fn render_layers_with_bg<F>(
    pass: &mut RenderPass,
    transform: &TransformParams,
    // TODO: maybe use AnyLayer here?
    layers: &[&dyn Layer],
    render_bg_cb: Option<F>,
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

    if render_bg_cb.is_some() {
        stencil_ref += 1;
    }

    for &layer in layers {
        render_items.push((layer, stencil_ref));
        stencil_ref += layer.get_stencil_bump();
    }

    for &(layer, stencil_ref) in render_items.iter().rev() {
        layer.render(pass, transform, stencil_ref, PassKind::Opaque);
    }

    if let Some(render_bg_cb) = render_bg_cb {
        render_bg_cb(pass, transform, orig_stencil_ref);
    }

    for &(layer, stencil_ref) in &render_items {
        layer.render(pass, transform, stencil_ref, PassKind::Transparent);
    }

    stencil_ref
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
        Some(render_layers_default_cb(color)),
        stencil_ref,
    );
}

pub fn render_layer(
    pass: &mut RenderPass,
    transform: &TransformParams,
    layer: &dyn Layer,
    color: FloatColor4,
    stencil_ref: u8,
) {
    render_layers_with_bg(
        pass,
        transform,
        &[layer],
        Some(render_layers_default_cb(color)),
        stencil_ref,
    );
}
