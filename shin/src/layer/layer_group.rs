use glam::vec3;
use itertools::Itertools;
use shin_core::{
    time::Ticks,
    vm::command::types::{LayerProperty, LayerbankId},
};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureTarget},
        vertices::{FloatColor4, PosVertex, UnormColor},
    },
    DepthStencilState, DrawPrimitive, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    StencilFunction, StencilMask, StencilOperation, StencilPipelineState, StencilState,
};

use crate::{
    layer::{
        new_drawable_layer::{NewDrawableLayerNeedsSeparatePass, NewDrawableLayerState},
        properties::LayerProperties,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        DrawableLayer, Layer, NewDrawableLayer, PreRenderContext, UserLayer,
    },
    update::{Updatable, UpdateContext},
};

#[derive(Clone)]
struct LayerItem {
    pub layerbank_id: LayerbankId,
    // this is probably how delayed removal is implemented?
    // pub some_countdown: Ticks,
    pub layer: UserLayer,
}

#[derive(Clone, Copy)]
struct LayerRenderItem {
    // TODO: maybe we should make the layers actually shareable with `Arc`s, yknow...
    pub layer_index: usize,
    pub stencil_ref_relative: u8,
}

#[derive(Clone)]
pub struct LayerGroup {
    layers: Vec<LayerItem>,

    stencil_bump: u8,
    layers_to_render: Vec<LayerRenderItem>,

    new_drawable_state: NewDrawableLayerState,
    mask_texture: Option<()>, // Option<GpuMaskTexture>,
    props: LayerProperties,
    label: String,
}

impl LayerGroup {
    // TODO: technically we need not only `UserLayer`s, but system layers sometimes too
    // Maybe I should bite the bullet and make an owning `AnyLayer` for all the types, or just use `Box<dyn Layer>`
    pub fn new(label: Option<String>) -> Self {
        Self {
            layers: vec![],
            stencil_bump: 0,
            layers_to_render: vec![],
            new_drawable_state: NewDrawableLayerState::new(),
            mask_texture: None,
            props: LayerProperties::new(),
            label: label.unwrap_or_else(|| "unnamed".to_string()),
        }
    }

    // NB: original game also accepts a `Wiper` argument which causes the layer to be wrapped in `LayerGroup::TransitionLayer`
    // umineko uses a different transition system, so this is not implemented
    pub fn add_layer(&mut self, layerbank_id: LayerbankId, layer: UserLayer) {
        match self
            .layers
            .binary_search_by_key(&layerbank_id, |item| item.layerbank_id)
        {
            Ok(index) => {
                self.layers[index].layer = layer;
            }
            Err(index) => {
                self.layers.insert(
                    index,
                    LayerItem {
                        layerbank_id,
                        layer,
                    },
                );
            }
        }
    }

    #[expect(unused)] // for future stuff
    pub fn remove_layer(&mut self, layerbank_id: LayerbankId, delay_time: Ticks) {
        if delay_time != Ticks::ZERO {
            // this is never used in umineko
            unimplemented!("LayerGroup::remove_layer: delay_time is not implemented");
        }

        match self
            .layers
            .binary_search_by_key(&layerbank_id, |item| item.layerbank_id)
        {
            Ok(index) => {
                self.layers.remove(index);
            }
            Err(_) => {
                // layerbank does not exist, swallow the error
            }
        }
    }

    #[expect(unused)] // for future stuff
    pub fn get_layer(&self, layerbank_id: LayerbankId) -> Option<&UserLayer> {
        self.layers
            .binary_search_by_key(&layerbank_id, |item| item.layerbank_id)
            .map(|index| &self.layers[index].layer)
            .ok()
    }

    #[expect(unused)] // for future stuff
    pub fn get_layer_mut(&mut self, layerbank_id: LayerbankId) -> Option<&mut UserLayer> {
        self.layers
            .binary_search_by_key(&layerbank_id, |item| item.layerbank_id)
            .map(|index| &mut self.layers[index].layer)
            .ok()
    }

    #[expect(unused)] // for future stuff
    pub fn get_used_layerbank_ids(&self) -> Vec<LayerbankId> {
        self.layers.iter().map(|item| item.layerbank_id).collect()
    }

    #[expect(unused)] // for future stuff
    pub fn has_wiper_for_layerbank(&self, _layerbank_id: LayerbankId) -> bool {
        false
    }

    #[expect(unused)] // for future stuff
    pub fn set_mask_texture(&mut self, mask_texture: Option<()>) {
        self.mask_texture = mask_texture;
    }

    pub fn needs_rendering(&self) -> bool {
        self.mask_texture.is_some() || self.layers.len() > 0
    }
}

struct LayerGroupNewDrawableSeparatePassDelegate {
    has_mask: bool,
}

impl NewDrawableLayerNeedsSeparatePass for LayerGroupNewDrawableSeparatePassDelegate {
    fn needs_separate_pass(&self, properties: &LayerProperties) -> bool {
        if self.has_mask {
            return true;
        }

        properties.get_clip_mode() != DrawableClipMode::None
            || properties.is_fragment_shader_nontrivial()
            || properties.is_blending_nontrivial()
    }
}

struct LayerGroupNewDrawableDelegate<'a> {
    layers: &'a [LayerItem],
    layers_to_render: Vec<LayerRenderItem>,
    mask_texture: &'a Option<()>,
}

impl NewDrawableLayerNeedsSeparatePass for LayerGroupNewDrawableDelegate<'_> {
    fn needs_separate_pass(&self, properties: &LayerProperties) -> bool {
        (LayerGroupNewDrawableSeparatePassDelegate {
            has_mask: self.mask_texture.is_some(),
        })
        .needs_separate_pass(properties)
    }
}

impl NewDrawableLayer for LayerGroupNewDrawableDelegate<'_> {
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        properties: &LayerProperties,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
        transform: &TransformParams,
    ) -> PassKind {
        let mut pass = RenderPass::new(
            context.pipeline_storage,
            context.dynamic_buffer,
            context.sampler_store,
            context.device,
            context.encoder,
            target,
            depth_stencil,
            None,
        );

        if !properties.is_visible() {
            pass.clear(Some(UnormColor::BLACK), None, None);
        } else {
            let mut self_transform = properties.get_transform_params();
            self_transform.compose_with(transform, properties.get_compose_flags());

            if let Some(_mask) = self.mask_texture {
                todo!()
            } else {
                pass.clear(Some(UnormColor::BLACK), Some(0), None);
            }

            for render_item in self.layers_to_render.iter().rev() {
                self.layers[render_item.layer_index].layer.render(
                    &mut pass,
                    &self_transform,
                    render_item.stencil_ref_relative,
                    PassKind::Opaque,
                );
            }
            for render_item in &self.layers_to_render {
                self.layers[render_item.layer_index].layer.render(
                    &mut pass,
                    &self_transform,
                    render_item.stencil_ref_relative,
                    PassKind::Transparent,
                );
            }
        }

        self.layers_to_render.clear();

        // it's very sus that it does that, considering you can apply alpha and stuff...
        // but I guess that's just how planes are...
        PassKind::Opaque
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
        // direct rendering is always done by the LayerGroup without relying on NewDrawableLayer
        unreachable!()
    }
}

impl Updatable for LayerGroup {
    fn update(&mut self, context: &UpdateContext) {
        self.props.update(context);
        self.new_drawable_state.update(context);

        for layer in &mut self.layers {
            layer.layer.update(context);
        }

        // this code handles the delayed layer removal
        // don't need it
        // for layer in &mut self.layers {
        //     if layer.layerbank_id < 0 {
        //         todo!("Run the timer and do something with TransitionLayers")
        //     }
        // }
    }
}

impl Layer for LayerGroup {
    fn get_stencil_bump(&self) -> u8 {
        self.stencil_bump
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        let mut layers = (0..self.layers.len()).into_iter().collect::<Vec<_>>();

        layers.sort_by(|&left, &right| {
            let get_values = |index: usize| {
                let layer = &self.layers[index];
                let prop = layer.layer.properties();

                let id = prop.get_layer_id();
                let render_position = prop.get_value(LayerProperty::RenderPosition);

                (render_position, id)
            };

            let (left_position, left_id) = get_values(left);
            let (right_position, right_id) = get_values(right);

            // if positions are close, compare by id
            if (left_position - right_position).abs() < f32::EPSILON {
                left_id.cmp(&right_id)
            } else {
                left_position.partial_cmp(&right_position).unwrap()
            }
        });

        // The original implementations handles `LayerGroup::TransitionLayer` here according to `Effectable` rules
        // This is not necessary for running umineko (it uses a different system for transition), so this is not implemented

        let props = self.properties();
        let mut self_transform = props.get_transform_params();
        self_transform.compose_with(transform, props.get_compose_flags());

        for &index in &layers {
            self.layers[index]
                .layer
                .pre_render(context, &self_transform);
            // NB: if the current layer is `Effectable` (like `LayerGroup::TransitionLayer`), we need to call a special pre-render function and pass the lower layers to it
            // This is not necessary for umineko
        }

        self.layers_to_render.clear();
        let mut layers_to_render = std::mem::replace(&mut self.layers_to_render, Vec::new());

        let mut stencil_value = 1;
        for &index in &layers {
            layers_to_render.push(LayerRenderItem {
                layer_index: index,
                stencil_ref_relative: stencil_value,
            });

            stencil_value += self.layers[index].layer.get_stencil_bump();
        }

        self.stencil_bump = stencil_value;

        let mut delegate = LayerGroupNewDrawableDelegate {
            layers: &self.layers,
            layers_to_render,
            mask_texture: &self.mask_texture,
        };

        self.new_drawable_state
            .pre_render(context, &self.props, &mut delegate, &self_transform);

        self.layers_to_render = delegate.layers_to_render;
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        if let Some(_tex) = self.new_drawable_state.get_prerendered_tex() {
            if let Some(_mask) = &self.mask_texture {
                // Self::render_with_mask
                todo!();
                return;
            } else {
                self.new_drawable_state.try_finish_indirect_render(
                    &self.props,
                    pass,
                    transform,
                    stencil_ref,
                    pass_kind,
                );
                return;
            }
        }

        let properties = &self.props;
        if !properties.is_visible() {
            return;
        }

        let mut self_transform = properties.get_transform_params();
        self_transform.compose_with(transform, properties.get_compose_flags());

        pass.push_debug(&format!(
            "LayerGroup[{}]/{}",
            &self.label,
            match pass_kind {
                PassKind::Opaque => {
                    "opaque"
                }
                PassKind::Transparent => {
                    "transparent"
                }
            }
        ));

        match pass_kind {
            PassKind::Opaque => {
                for &LayerRenderItem {
                    layer_index,
                    stencil_ref_relative,
                } in self.layers_to_render.iter().rev()
                {
                    self.layers[layer_index].layer.render(
                        pass,
                        &self_transform,
                        stencil_ref + stencil_ref_relative,
                        pass_kind,
                    )
                }

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

                pass.push_debug("clear");
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
                                    stencil_read_mask: StencilMask::All,
                                    stencil_write_mask: StencilMask::All,
                                },
                                stencil_reference: stencil_ref,
                            },
                        })
                        .build(
                            RenderProgramWithArguments::Clear {
                                vertices: VertexSource::VertexData { vertices },
                                color: FloatColor4::BLACK,
                            },
                            DrawPrimitive::Triangles,
                        ),
                );
                pass.pop_debug();
            }
            PassKind::Transparent => {
                for &LayerRenderItem {
                    layer_index,
                    stencil_ref_relative,
                } in &self.layers_to_render
                {
                    self.layers[layer_index].layer.render(
                        pass,
                        &self_transform,
                        stencil_ref + stencil_ref_relative,
                        pass_kind,
                    )
                }
            }
        }

        pass.pop_debug();
    }
}

impl DrawableLayer for LayerGroup {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
