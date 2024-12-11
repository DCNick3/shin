use glam::{vec3, Mat4};
use shin_core::vm::command::types::{PlaneId, PLANES_COUNT};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureTarget},
        vertices::{FloatColor4, PosVertex},
    },
    DepthStencilState, DrawPrimitive, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    StencilFunction, StencilMask, StencilOperation, StencilPipelineState, StencilState,
};

use crate::{
    layer::{
        new_drawable_layer::{NewDrawableLayerNeedsSeparatePass, NewDrawableLayerState},
        properties::LayerProperties,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        DrawableLayer, Layer, LayerGroup, NewDrawableLayer, PreRenderContext,
    },
    update::{Updatable, UpdateContext},
};

#[derive(Clone, Copy)]
struct LayerRenderItem {
    pub layer_index: PlaneId,
    pub stencil_ref_relative: u8,
}

#[derive(Clone)]
pub struct PageLayer {
    planes: Vec<LayerGroup>,

    stencil_bump: u8,
    needs_force_blending: bool,
    layers_to_render: Vec<LayerRenderItem>,

    new_drawable_state: NewDrawableLayerState,
    props: LayerProperties,
    label: String,
}

impl PageLayer {
    pub fn new(plane_count: usize, label: Option<String>) -> Self {
        // can't support more, the `PlaneId` type is specialized to 4 planes
        // ideally I would like to only support 4 planes, but `BustupMode` at least uses 1
        assert!(plane_count <= PLANES_COUNT);

        let label = label.unwrap_or_else(|| "unnamed".to_string());

        Self {
            planes: (0..plane_count)
                .map(|i| LayerGroup::new(Some(format!("{}/LayerGroup{}", label, i))))
                .collect(),
            stencil_bump: 1,
            needs_force_blending: false,
            layers_to_render: vec![],
            new_drawable_state: NewDrawableLayerState::new(),
            props: LayerProperties::new(),
            label,
        }
    }

    pub fn get_plane(&self, index: PlaneId) -> &LayerGroup {
        &self.planes[index.raw() as usize]
    }

    pub fn get_plane_mut(&mut self, index: PlaneId) -> &mut LayerGroup {
        &mut self.planes[index.raw() as usize]
    }

    fn get_new_drawable_delegate(&self) -> PageLayerNewDrawableDelegate {
        PageLayerNewDrawableDelegate
    }
}

struct PageLayerNewDrawableDelegate;

impl NewDrawableLayerNeedsSeparatePass for PageLayerNewDrawableDelegate {
    fn needs_separate_pass(&self, properties: &LayerProperties) -> bool {
        properties.get_clip_mode() != DrawableClipMode::None
            || properties.is_fragment_shader_nontrivial()
            || properties.is_blending_nontrivial()
    }
}

impl NewDrawableLayer for PageLayerNewDrawableDelegate {
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        properties: &LayerProperties,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
        transform: &TransformParams,
    ) -> PassKind {
        todo!()
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
        // direct rendering is always done by the PageLayer without relying on NewDrawableLayer
        unreachable!()
    }
}

impl Updatable for PageLayer {
    fn update(&mut self, context: &UpdateContext) {
        self.props.update(context);
        self.new_drawable_state.update(context);

        for plane in self.planes.iter_mut() {
            plane.update(context);
        }
    }
}

impl Layer for PageLayer {
    fn get_stencil_bump(&self) -> u8 {
        self.stencil_bump
    }

    fn pre_render(&mut self, context: &mut PreRenderContext, transform: &TransformParams) {
        let props = &self.props;
        let mut self_transform = props.get_transform_params();
        self_transform.compose_with(transform, props.get_compose_flags());

        let mut layer_stack = Vec::new();

        for index in 0..self.planes.len() {
            let layer_group = &mut self.planes[index];

            layer_group.pre_render(context, &self_transform);
            if layer_group.needs_rendering() {
                // if a layer is rendered directly, we won't be able to see anything below it
                if self
                    .new_drawable_state
                    .is_rendered_directly(props, &self.get_new_drawable_delegate())
                {
                    layer_stack.clear();
                }

                layer_stack.push(index);
            }
        }

        self.layers_to_render.clear();
        self.needs_force_blending = props.is_blending_nontrivial();
        let mut stencil_value = 1;

        for index in layer_stack {
            self.layers_to_render.push(LayerRenderItem {
                layer_index: PlaneId::new(index as _),
                stencil_ref_relative: stencil_value,
            });

            stencil_value += self.planes[index].get_stencil_bump();
        }
        self.stencil_bump = stencil_value;

        let mut delegate = self.get_new_drawable_delegate();

        self.new_drawable_state
            .pre_render(context, &self.props, &mut delegate, &self_transform);
    }

    fn render(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        let props = self.properties();
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

        let mut self_transform = props.get_transform_params();
        self_transform.compose_with(transform, props.get_compose_flags());

        pass.push_debug(&format!(
            "PageLayer[{}]/{}",
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
                    self.planes[layer_index.raw() as usize].render(
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
                    self.planes[layer_index.raw() as usize].render(
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

impl DrawableLayer for PageLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
