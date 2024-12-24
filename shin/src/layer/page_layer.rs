use glam::vec3;
use shin_core::{
    primitives::color::{FloatColor4, UnormColor},
    vm::command::types::{PlaneId, PLANES_COUNT},
};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{
        buffer::VertexSource,
        texture::{DepthStencilTarget, TextureTarget},
        vertices::PosVertex,
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
    update::{AdvUpdatable, AdvUpdateContext},
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
}

impl PageLayer {
    pub fn new(plane_count: usize) -> Self {
        // can't support more, the `PlaneId` type is specialized to 4 planes
        // ideally I would like to only support 4 planes, but `BustupMode` at least uses 1
        assert!(plane_count <= PLANES_COUNT);

        Self {
            planes: (0..plane_count)
                .map(|i| LayerGroup::new(Some(format!("PageLayer/Plane{}", i))))
                .collect(),
            stencil_bump: 1,
            needs_force_blending: false,
            layers_to_render: vec![],
            new_drawable_state: NewDrawableLayerState::new(),
            props: LayerProperties::new(),
        }
    }

    pub fn get_plane(&self, index: PlaneId) -> &LayerGroup {
        &self.planes[index.raw() as usize]
    }

    pub fn get_plane_mut(&mut self, index: PlaneId) -> &mut LayerGroup {
        &mut self.planes[index.raw() as usize]
    }
}
struct PageLayerNewDrawableSeparatePassDelegate;

impl NewDrawableLayerNeedsSeparatePass for PageLayerNewDrawableSeparatePassDelegate {
    fn needs_separate_pass(&self, properties: &LayerProperties) -> bool {
        properties.get_clip_mode() != DrawableClipMode::None
            || properties.is_fragment_shader_nontrivial()
            || properties.is_blending_nontrivial()
    }
}

struct PageLayerNewDrawableDelegate<'a> {
    planes: &'a [LayerGroup],
    layers_to_render: Vec<LayerRenderItem>,
}

impl NewDrawableLayerNeedsSeparatePass for PageLayerNewDrawableDelegate<'_> {
    fn needs_separate_pass(&self, properties: &LayerProperties) -> bool {
        PageLayerNewDrawableSeparatePassDelegate.needs_separate_pass(properties)
    }
}

impl NewDrawableLayer for PageLayerNewDrawableDelegate<'_> {
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

            pass.clear(Some(UnormColor::BLACK), Some(0), None);

            for render_item in self.layers_to_render.iter().rev() {
                self.planes[render_item.layer_index.raw() as usize].render(
                    &mut pass,
                    &self_transform,
                    render_item.stencil_ref_relative,
                    PassKind::Opaque,
                );
            }
            for render_item in &self.layers_to_render {
                self.planes[render_item.layer_index.raw() as usize].render(
                    &mut pass,
                    &self_transform,
                    render_item.stencil_ref_relative,
                    PassKind::Transparent,
                );
            }
        }

        self.layers_to_render.clear();

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
        // direct rendering is always done by the PageLayer without relying on NewDrawableLayer
        unreachable!()
    }
}

impl AdvUpdatable for PageLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
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
        let self_transform = props.get_composed_transform_params(transform);

        let mut layer_stack = Vec::new();

        for index in 0..self.planes.len() {
            let layer_group = &mut self.planes[index];

            layer_group.pre_render(context, &self_transform);
            if layer_group.needs_rendering() {
                // if a layer is rendered directly, we won't be able to see anything below it
                if self
                    .new_drawable_state
                    .is_rendered_opaquely(props, &PageLayerNewDrawableSeparatePassDelegate)
                {
                    layer_stack.clear();
                }

                layer_stack.push(index);
            }
        }

        self.layers_to_render.clear();
        let mut layers_to_render = std::mem::take(&mut self.layers_to_render);
        self.needs_force_blending = props.is_blending_nontrivial();
        let mut stencil_value = 1;

        for index in layer_stack {
            layers_to_render.push(LayerRenderItem {
                layer_index: PlaneId::new(index as _),
                stencil_ref_relative: stencil_value,
            });

            stencil_value += self.planes[index].get_stencil_bump();
        }
        self.stencil_bump = stencil_value;

        let mut delegate = PageLayerNewDrawableDelegate {
            planes: &self.planes,
            layers_to_render,
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

        let self_transform = props.get_composed_transform_params(transform);

        pass.push_debug(&format!(
            "PageLayer/{}",
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
